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
    /// The position of every media of this account. See T-127.
    ///
    /// **A row that the program cannot read must not take the other rows
    /// away** (T-41). The server gives a whole number for one media and a
    /// fraction for the next one, therefore each row reads by itself here.
    #[serde(default, rename = "mediaProgress")]
    media_progress: Vec<serde_json::Value>,
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
    Ok(the_account_of_the_token(client).await?.0)
}

/// Asks the server for the account of the token: what it may do, and the
/// position of every media of it. See T-110 and T-127.
///
/// **One request holds every position.** The start of the program asked
/// `GET /api/me/progress/:id` for each media of the Home view: a list of 29
/// media of a server of 500 milliseconds took **2.1 seconds** of a start of
/// 3.8. `GET /api/me` gives `mediaProgress` for every media of the account,
/// and the program asks that endpoint for the permissions already.
pub async fn the_account_of_the_token(
    client: &ApiClient,
) -> Result<(TheAccount, Vec<crate::api::me::get_media_progress::Root>), ApiError> {
    let me: Me = client.get_json("/api/me").await?;

    let the_positions = the_positions_of_the_answer(me.media_progress);

    Ok((
        TheAccount {
            kind: me.kind.unwrap_or_default(),
            permissions: me.permissions.unwrap_or_default(),
        },
        the_positions,
    ))
}

/// Reads the rows of `mediaProgress` of the answer of `GET /api/me`.
///
/// **A row that does not read takes a line of the log and no more.** The
/// position of one media stands on the line of that media, therefore the
/// program keeps every row that reads and it stops for none of them. The row
/// that does not read names no media (`get_media_progress::Root` asks for
/// `libraryItemId` alone), therefore it belongs to no line of any view and no
/// view can say a word of it. See T-177.
///
/// The function is pure, therefore a test needs no server.
pub fn the_positions_of_the_answer(
    rows: Vec<serde_json::Value>,
) -> Vec<crate::api::me::get_media_progress::Root> {
    let mut the_positions = Vec::new();

    for row in rows {
        match serde_json::from_value::<crate::api::me::get_media_progress::Root>(row) {
            Ok(one) => the_positions.push(one),
            Err(error) => log::warn!("[app] a position of the account does not read: {}", error),
        }
    }

    the_positions
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

    /// **The positions of a server of another version read.** A measurement of
    /// 2026-08-14 took `mediaItemId` and `mediaItemType` out of the answer of
    /// `GET /api/me` with `docs/harness/a_field_of_the_answer_goes_away.py`:
    /// the 20 rows of the account each said "missing field `mediaItemId`" in
    /// the log, and the Home view of the program then showed the position of no
    /// media at all. See T-177.
    #[test]
    fn the_positions_of_a_server_of_another_version_read() {
        let me: Me = serde_json::from_value(serde_json::json!({
            "id": "a-user",
            "username": "toutuitest",
            "mediaProgress": [
                {
                    "id": "a-row",
                    "userId": "a-user",
                    "libraryItemId": "a-book",
                    "episodeId": null,
                    "duration": 1800.0,
                    "progress": 0.5,
                    "currentTime": 900.0,
                    "isFinished": false,
                    "hideFromContinueListening": false,
                    "ebookLocation": null,
                    "ebookProgress": 0,
                    "lastUpdate": 1i64,
                    "startedAt": 1i64,
                    "finishedAt": null
                },
                {
                    "id": "another-row",
                    "userId": "a-user",
                    "progress": 0.5
                }
            ]
        }))
        .expect("the answer of the server must read");

        let positions = the_positions_of_the_answer(me.media_progress);

        // The row of the media reads, and the row that names no media does not
        // take the other one away.
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].library_item_id, "a-book");
        assert_eq!(positions[0].progress, 0.5);
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
