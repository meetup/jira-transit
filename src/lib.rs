extern crate afterparty;
extern crate goji;
#[macro_use]
extern crate log;
extern crate hubcaps;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use afterparty::{Delivery, Event, Hook};

mod config;
mod github;
mod jira;
mod directive;

// re-exports
pub use github::{DefaultGithub, Github};
pub use jira::{DefaultJira, Jira};
pub use config::Config;
pub use directive::Directive;

use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::sync::{Arc, Mutex};

/// a pull is a reference to a github pull request for a given repo
#[derive(Debug, Clone)]
pub struct Pull {
    /// number of pull request
    pub number: u64,
    /// repo slug in owner/repo format
    pub repo_slug: String,
}

impl Pull {
    pub fn new<R>(num: u64, repo: R) -> Pull
        where R: Into<String>
    {
        Pull {
            number: num,
            repo_slug: repo.into(),
        }
    }
}

/// the primary orchestator for handling github webhooks
pub struct Transit {
    sender: Arc<Mutex<Sender<Pull>>>,
}

impl Transit {
    pub fn new<'a>(github: Box<Github>, jira: Box<Jira>) -> Transit {
        let (tx, rx) = channel();
        // start work queue
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(pull) => Self::merged(pull, &github, &jira),
                    _ => break,

                }
            }
        });
        Transit { sender: Arc::new(Mutex::new(tx)) }
    }

    pub fn parse_content(content: github::Content) -> Vec<Directive> {
        let github::Content { commits, comments } = content;
        let commit_directives = commits.iter().fold(vec![], |mut result, commit| {
            for d in directive::parse(commit.as_ref()) {
                result.push(d)
            }
            result
        });
        comments.iter().fold(commit_directives, |mut result, comment| {
            for d in directive::parse(comment.as_ref()) {
                result.push(d)
            }
            result
        })
    }

    /// process a pull request
    fn merged(pull: Pull, github: &Box<Github>, jira: &Box<Jira>) {
        info!("pull {:?}", pull);
        let content = github.content(pull.clone());
        // parse directives
        let directives = Self::parse_content(content);
        info!("directives for pr {} in repo {} {:?}",
              pull.number,
              pull.repo_slug,
              directives);
        // attempt transition
        jira.transition(directives)
    }
}

impl Hook for Transit {
    fn handle(&self, delivery: &Delivery) {
        info!("recv {} delivery {}", delivery.event, delivery.id);
        match delivery.payload {
            /// handle all merged pull request events
            Event::PullRequest { ref action, ref pull_request, ref repository, .. }
                if action == "closed" && pull_request.merged => {
                // TODO to common branch
                // enqueue work
                let _ = self.sender
                    .lock()
                    .unwrap()
                    .send(Pull::new(pull_request.number, repository.full_name.clone()));
            }
            _ => (), // pass on all other wehbook events
        }
    }
}
