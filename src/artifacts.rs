extern crate reqwest;
extern crate regex;

use self::regex::Regex;

/// An artifact as can be found on the artifact server
#[derive(Clone, Debug)]
pub struct Artifact {
    /// The URL for the artifact folder.
    pub url: String,
    /// The artifact publishing number. (Higher is more recent)
    pub num: u16,
    /// The hash of the artifact.
    pub hash: String,
}

/// Gets a list of artifacts found on the artifact server provided.
/// If the artifact server is invalid (or is updated and this crate
/// is not), an empty list will always be returned.
pub fn get_artifacts(url: String) -> Vec<Artifact> {
    if let Ok(mut body) = reqwest::get(&url) {
        if let Ok(body) = body.text() {
            let re = Regex::new(r"(\d+)\-([\da-f]+)").unwrap();
            let mut versions: Vec<Artifact> = Vec::new();
            let mut nums_already: Vec<u16> = Vec::new();

            for capture in re.captures_iter(&body) {
                let num = (&capture[1]).parse::<u16>().unwrap();
                if !nums_already.contains(&num) {
                    nums_already.push(num);
                    versions.push(Artifact {
                        url: format!("{}{}-{}/", url, &capture[1], &capture[2]),
                        num: num,
                        hash: (&capture[2]).to_string(),
                    });
                }
            }
            return versions;
        }
    }
    vec![]
}
