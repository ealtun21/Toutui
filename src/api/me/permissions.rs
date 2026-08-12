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
    /// The type of the account: `root`, `admin`, `user`, or `guest`.
    #[serde(default, rename = "type")]
    kind: Option<String>,
}

/// The account of the user, as the settings show it. See T-110.
#[derive(Debug, Clone, Default)]
pub struct TheAccount {
    /// The type of the account, as the server names it. It is empty for a
    /// server that names none.
    pub kind: String,
    /// What this account may do.
    pub permissions: Permissions,
}

/// Gives the lines of the account, for the settings. See T-110.
///
/// **The program showed no permission and no type**, therefore a user whose
/// account may not download read the message of the key `D` and nothing else.
/// This screen says what the account may do before the user presses a key.
///
/// The function is pure, therefore a test needs no server.
pub fn the_lines_of_the_account(name: &str, account: &TheAccount) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    lines.push(match account.kind.trim() {
        "" => format!("The account {}. The server names no type.", name),
        kind => format!("The account {}, of the type {}.", name, kind),
    });

    lines.push(String::new());

    // Each line names one permission and the work of the program that needs it.
    // A permission that the program never uses stands in no line: a user reads
    // what changes their work only.
    lines.push(the_line_of_a_permission(
        account.permissions.download,
        "make a copy of a media on the disk (the key D)",
    ));
    lines.push(the_line_of_a_permission(
        account.permissions.update,
        "give a collection or a playlist a new name, a new description, and a new sequence",
    ));
    lines.push(the_line_of_a_permission(
        account.permissions.delete,
        "remove a collection or a playlist",
    ));

    lines
}

/// Makes one line of a permission, in the words of a user.
fn the_line_of_a_permission(may: bool, what: &str) -> String {
    match may {
        true => format!("You may {}.", what),
        false => format!("You may not {}.", what),
    }
}

/// Asks the server what this account may do.
///
/// A server that does not answer gives every permission. The program then
/// works as it did before this function, and the server still refuses what it
/// must refuse.
pub async fn get_permissions(client: &ApiClient) -> Result<TheAccount, ApiError> {
    let me: Me = client.get_json("/api/me").await?;

    Ok(TheAccount {
        kind: me.kind.unwrap_or_default(),
        permissions: me.permissions.unwrap_or_default(),
    })
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
            "type": "root",
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

        assert_eq!(me.kind.as_deref(), Some("root"));

        let permissions = me.permissions.expect("the answer holds the permissions");
        assert!(permissions.download);
        assert!(!permissions.update);
        assert!(!permissions.delete);
        assert!(permissions.upload);
    }

    /// **The settings say the type of the account and every permission that
    /// changes the work of the program**, in the words of a user. See T-110.
    #[test]
    fn the_settings_say_what_the_account_may_do() {
        let account = TheAccount {
            kind: "root".to_string(),
            permissions: Permissions {
                download: true,
                update: false,
                delete: false,
                upload: true,
            },
        };

        let lines = the_lines_of_the_account("toutuitest", &account);
        let text = lines.join("\n");

        assert!(
            text.contains("The account toutuitest, of the type root."),
            "{}",
            text
        );
        assert!(
            text.contains("You may make a copy of a media on the disk"),
            "{}",
            text
        );
        assert!(
            text.contains("You may not give a collection or a playlist a new name"),
            "{}",
            text
        );
        assert!(
            text.contains("You may not remove a collection or a playlist."),
            "{}",
            text
        );

        // A server that names no type must give no empty sentence.
        let no_type = TheAccount::default();
        let lines = the_lines_of_the_account("toutuitest", &no_type);
        assert!(
            lines[0].contains("The server names no type."),
            "{:?}",
            lines
        );

        // An absent permission means "yes", therefore the default account may
        // do every work of the program.
        let text = lines.join("\n");
        assert!(!text.contains("You may not"), "{}", text);
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
