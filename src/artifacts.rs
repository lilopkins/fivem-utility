use regex::Regex;

use std::cmp::Ordering;

/// An artifact as can be found on the artifact server
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Artifact {
    /// The URL for the artifact folder.
    pub url: String,
    /// The artifact publishing number. (Higher is more recent)
    pub num: u16,
    /// The hash of the artifact.
    pub hash: String,
}

impl Ord for Artifact {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num.cmp(&other.num)
    }
}

impl PartialOrd for Artifact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.num.cmp(&other.num))
    }
}

/// An instance of an artifact server. This caches responses to provide the quickest response it can each time.
#[derive(Clone, Debug)]
pub struct ArtifactServer<'a> {
    url: &'a str,
    body: Option<String>,
    latest: Option<u16>,
    artifacts: Option<Vec<Artifact>>,
}

impl<'a> ArtifactServer<'a> {
    /// Create a new Artifact server with caching
    pub fn new(url: &'a str) -> Self {
        Self {
            url: url,
            body: None,
            artifacts: None,
            latest: None,
        }
    }

    fn get_body(&mut self) -> bool {
        if let Ok(body) = reqwest::blocking::get(self.url) {
            if let Ok(body) = body.text() {
                self.body = Some(body);
                return true;
            }
        }
        eprintln!("!! Failed to fetch artifact server content !!");
        false
    }

    /// Gets a list of artifacts found on the artifact server.
    /// If the artifact server is invalid (or is updated and this crate
    /// is not), an empty list will always be returned.
    pub fn get_artifacts(&mut self) -> Vec<Artifact> {
        if let Some(arts) = &self.artifacts {
            return arts.clone();
        }
        if self.body == None {
            self.get_body();
        }
        if let Some(body) = &self.body {
            let re = Regex::new(r"(\d+)\-([\da-f]+)/fx\.tar\.xz").unwrap();
            let mut versions: Vec<Artifact> = Vec::new();
            let mut nums_already: Vec<u16> = Vec::new();

            for capture in re.captures_iter(&body) {
                let num = (&capture[1]).parse::<u16>().unwrap();
                if !nums_already.contains(&num) {
                    nums_already.push(num);
                    versions.push(Artifact {
                        url: format!("{}{}-{}/", self.url, &capture[1], &capture[2]),
                        num: num,
                        hash: (&capture[2]).to_string(),
                    });
                }
            }
            self.artifacts = Some(versions);
            return self.artifacts.clone().unwrap();
        }
        vec![]
    }

    /// Gets the number of the latest recommended artifact.
    /// If the artifact server is invalid (or is updated and this crate
    /// is not), zero (`0u16`) will be returned
    pub fn get_latest_version_num(&mut self) -> u16 {
        if let Some(latest) = self.latest {
            return latest;
        }
        if self.body == None {
            self.get_body();
        }
        if let Some(body) = &self.body {
            let re = Regex::new(r"RECOMMENDED \((\d+)\)").unwrap();
            
            for capture in re.captures_iter(&body) {
                let num = (&capture[1]).parse::<u16>().unwrap();
                self.latest = Some(num);
                return num;
            }
        }
        0u16
    }

    /// Linear search for an artifact
    pub fn get_artifact(&mut self, num: u16) -> Option<Artifact> {
        if self.artifacts == None {
            self.get_artifacts();
        }

        for ar in self.artifacts.as_ref().unwrap_or(&Vec::new()) {
            if ar.num == num {
                return Some(ar.clone());
            }
        }
        None
    }
}
