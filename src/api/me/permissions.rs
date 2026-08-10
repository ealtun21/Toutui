//! What the account of the user may do. See T-24.
//!
//! `GET /api/me` gives nine permissions. The program never read them, and it
//! sent a request that the server refused. The user then saw the error of the
//! server, and not a sentence that says what is wrong.
//!
//! A measurement against an Audiobookshelf 2.36.0 on 2026-08-11 gives these
//! names: `download`, `update`, `delete`, `upload`, `createEreader`,
//! `accessAllLibraries`, `accessAllTags`, `accessExplicitContent`, and
//! `selectedTagsNotAccessible`.
//!
//! **An absent permission means "yes".** A server that gives no permission at
//! all must not stop the user from working. The program then behaves as it did
//! before, and the server still refuses what it must refuse.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Permissions {
    #[serde(default = "yes")]
    pub download: bool,
    #[serde(default = "yes")]
    pub update: bool,
    #[serde(default = "yes")]
    pub delete: bool,
    #[serde(default = "yes")]
    pub upload: bool,
}

fn yes() -> bool {
    true
}

impl Default for Permissions {
    fn default() -> Self {
        Permissions {
            download: true,
            update: true,
            delete: true,
            upload: true,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct Me {
    #[serde(default)]
    permissions: Option<Permissions>,
}

/// Asks the server what this account may do.
///
/// A server that does not answer gives every permission. The program then
/// works as it did before this function, and the server still refuses what it
/// must refuse.
pub async fn get_permissions(client: &ApiClient) -> Result<Permissions, ApiError> {
    let me: Me = client.get_json("/api/me").await?;

    Ok(me.permissions.unwrap_or_default())
}

/// Gives the sentence for a user who may not download.
pub fn no_download() -> &'static str {
    "Your account cannot download a media. Ask the person who holds the server."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_answer_of_a_real_server_reads() {
        let me: Me = serde_json::from_value(serde_json::json!({
            "id": "a-user",
            "username": "toutuitest",
            "permissions": {
                "download": true,
                "update": false,
                "delete": false,
                "upload": true,
                "createEreader": true,
                "accessAllLibraries": true,
                "accessAllTags": true,
                "accessExplicitContent": true,
                "selectedTagsNotAccessible": false
            }
        }))
        .expect("the answer of the server must read");

        let permissions = me.permissions.expect("the answer holds the permissions");
        assert!(permissions.download);
        assert!(!permissions.update);
        assert!(!permissions.delete);
        assert!(permissions.upload);
    }

    #[test]
    fn an_answer_with_no_permission_gives_every_permission() {
        let me: Me = serde_json::from_value(serde_json::json!({ "username": "toutuitest" }))
            .expect("the answer must read");

        let permissions = me.permissions.unwrap_or_default();
        assert!(permissions.download);
        assert!(permissions.update);
    }

    #[test]
    fn a_permission_that_the_server_does_not_name_is_a_yes() {
        // A server of a different version can name fewer permissions. The
        // program must not stop the user for a name that it did not find.
        let me: Me = serde_json::from_value(serde_json::json!({
            "permissions": { "download": false }
        }))
        .expect("the answer must read");

        let permissions = me.permissions.expect("the answer holds the permissions");
        assert!(!permissions.download);
        assert!(permissions.update);
        assert!(permissions.delete);
    }
}
