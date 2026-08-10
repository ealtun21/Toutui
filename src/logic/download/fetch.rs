//! Gets the audio files of a book and writes them to the disk.
//!
//! The module uses `GET /api/items/:id/file/:ino/download`. This endpoint
//! gives the true audio file and it accepts the header `Range`. Therefore a
//! download continues after an interruption.
//!
//! The module does not hold a complete file in the memory. It writes each
//! block of data to the disk.

use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

use super::plan::{resume_from, AudioFilePlan, DownloadPlan, Resume};
use super::progress::{DownloadProgress, DownloadState, ProgressMap};

/// Gets all the audio files of one book.
///
/// The function writes each file to `dest_dir`. It writes to a file with the
/// name `.part` first. When the file is complete, the function changes the
/// name. Therefore a file without `.part` is always complete.
///
/// The function keeps the `.part` file when an error occurs. The next attempt
/// continues from that point.
///
/// The function takes a `&reqwest::Client` now. Sub-project 1 gives the type
/// `ApiClient`. Then this parameter becomes `&ApiClient`.
pub async fn fetch_item(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    plan: &DownloadPlan,
    dest_dir: &Path,
    progress: ProgressMap,
) -> Result<Vec<PathBuf>, String> {
    let total_bytes = plan.total_bytes();

    write_progress(
        &progress,
        |state| {
            state.file_index = 1;
            state.file_count = plan.files.len();
            state.bytes_done = 0;
            state.bytes_total = total_bytes;
            state.state = DownloadState::Running;
        },
        plan,
    );

    if let Err(error) = tokio::fs::create_dir_all(dest_dir).await {
        return Err(fail(
            &progress,
            plan,
            format!("cannot make the directory: {error}"),
        ));
    }

    let mut paths = Vec::with_capacity(plan.files.len());
    let mut done_bytes: u64 = 0;

    for (number, file) in plan.files.iter().enumerate() {
        write_progress(
            &progress,
            |state| {
                state.file_index = number + 1;
                state.bytes_done = done_bytes;
            },
            plan,
        );

        let target = dest_dir.join(file.disk_name());
        let part = part_path(&target);

        match fetch_one(
            client, base_url, token, file, &target, &part, done_bytes, &progress, plan,
        )
        .await
        {
            Ok(path) => paths.push(path),
            Err(error) => return Err(fail(&progress, plan, error)),
        }

        done_bytes += file.size;

        write_progress(&progress, |state| state.bytes_done = done_bytes, plan);
    }

    write_progress(
        &progress,
        |state| {
            state.bytes_done = total_bytes;
            state.state = DownloadState::Finished;
        },
        plan,
    );

    Ok(paths)
}

/// Gets one audio file and writes it to the disk.
///
/// The parameter `done_bytes` is the number of bytes of the files before this
/// file. The function adds the bytes of this file to that number.
#[allow(clippy::too_many_arguments)]
async fn fetch_one(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    file: &AudioFilePlan,
    target: &Path,
    part: &Path,
    done_bytes: u64,
    progress: &ProgressMap,
    plan: &DownloadPlan,
) -> Result<PathBuf, String> {
    // The file on the disk tells the function where to start.
    let resume = resume_from(part, file.size)
        .map_err(|error| format!("cannot read {}: {error}", part.display()))?;

    if let Resume::Complete = resume {
        rename(part, target).await?;
        return Ok(target.to_path_buf());
    }

    let Resume::From(mut have) = resume else {
        unreachable!("the enum has two values")
    };

    // The file is already on the disk with the correct size. No request is
    // necessary.
    if let Ok(metadata) = tokio::fs::metadata(target).await {
        if metadata.len() == file.size {
            return Ok(target.to_path_buf());
        }
    }

    let url = format!(
        "{}/api/items/{}/file/{}/download",
        base_url.trim_end_matches('/'),
        plan.item_id,
        file.ino
    );

    let mut request = client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"));

    if have > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={have}-"));
    }

    let mut response = request
        .send()
        .await
        .map_err(|error| format!("the request for {} failed: {error}", file.filename))?;

    if !response.status().is_success() {
        return Err(format!(
            "the server answered {} for {}",
            response.status(),
            file.filename
        ));
    }

    // A 206 answer continues the file. A 200 answer gives the full file, and
    // the function writes from the start.
    let partial = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;

    if !partial {
        have = 0;
    }

    let mut handle = if partial && have > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(part)
            .await
    } else {
        tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(part)
            .await
    }
    .map_err(|error| format!("cannot open {}: {error}", part.display()))?;

    let mut written = have;

    write_progress(
        progress,
        |state| state.bytes_done = done_bytes + written,
        plan,
    );

    // The function reads one block at a time. Therefore a book of 700 MB does
    // not fill the memory.
    loop {
        let chunk = response
            .chunk()
            .await
            .map_err(|error| format!("the transfer of {} stopped: {error}", file.filename))?;

        let Some(chunk) = chunk else { break };

        if chunk.is_empty() {
            continue;
        }

        handle
            .write_all(&chunk)
            .await
            .map_err(|error| format!("cannot write {}: {error}", part.display()))?;

        written += chunk.len() as u64;

        // The function changes the progress one time for each block, and not
        // one time for each byte. It holds the lock for a short time.
        write_progress(
            progress,
            |state| state.bytes_done = done_bytes + written,
            plan,
        );
    }

    handle
        .flush()
        .await
        .map_err(|error| format!("cannot write {}: {error}", part.display()))?;
    drop(handle);

    if written != file.size {
        return Err(format!(
            "the server sent {written} bytes for {}, but the file has {} bytes",
            file.filename, file.size
        ));
    }

    rename(part, target).await?;

    Ok(target.to_path_buf())
}

/// Gives the name of the file that is not complete.
fn part_path(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_os_string();
    name.push(".part");
    PathBuf::from(name)
}

/// Changes the name of a complete file.
async fn rename(part: &Path, target: &Path) -> Result<(), String> {
    tokio::fs::rename(part, target)
        .await
        .map_err(|error| format!("cannot rename {}: {error}", part.display()))
}

/// Writes the cause of the error to the progress and gives the text back.
fn fail(progress: &ProgressMap, plan: &DownloadPlan, message: String) -> String {
    write_progress(
        progress,
        |state| state.state = DownloadState::Failed(message.clone()),
        plan,
    );
    message
}

/// Changes the progress of this item. The function holds the lock for a short
/// time only.
fn write_progress<F>(progress: &ProgressMap, change: F, plan: &DownloadPlan)
where
    F: FnOnce(&mut DownloadProgress),
{
    let Ok(mut map) = progress.write() else {
        return;
    };

    let state = map
        .entry(plan.key.clone())
        .or_insert_with(|| DownloadProgress {
            key: plan.key.clone(),
            title: plan.title.clone(),
            file_index: 1,
            file_count: plan.files.len(),
            bytes_done: 0,
            bytes_total: plan.total_bytes(),
            state: DownloadState::Running,
        });

    change(state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Makes a plan with the given files. Each file has the size of its body.
    fn plan_of(files: &[(u32, &str, &str, u64)]) -> DownloadPlan {
        DownloadPlan {
            item_id: "item-1".to_string(),
            key: "item-1".to_string(),
            title: "A Book".to_string(),
            author: "An Author".to_string(),
            files: files
                .iter()
                .map(|(index, ino, filename, size)| AudioFilePlan {
                    index: *index,
                    ino: (*ino).to_string(),
                    filename: (*filename).to_string(),
                    size: *size,
                    duration: 1.0,
                })
                .collect(),
        }
    }

    fn map() -> ProgressMap {
        Arc::new(RwLock::new(HashMap::new()))
    }

    /// Gives the progress of the item.
    fn state_of(progress: &ProgressMap) -> DownloadProgress {
        progress.read().unwrap().get("item-1").unwrap().clone()
    }

    #[tokio::test]
    async fn a_new_download_writes_every_file() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        Mock::given(method("GET"))
            .and(path("/api/items/item-1/file/10/download"))
            .and(header("authorization", "Bearer secret"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 100]))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/items/item-1/file/11/download"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'b'; 50]))
            .mount(&server)
            .await;

        let plan = plan_of(&[(1, "10", "one.mp3", 100), (2, "11", "two.mp3", 50)]);
        let progress = map();

        let paths = fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            progress.clone(),
        )
        .await
        .unwrap();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], dir.path().join("001 - one.mp3"));
        assert_eq!(paths[1], dir.path().join("002 - two.mp3"));
        assert_eq!(std::fs::read(&paths[0]).unwrap(), vec![b'a'; 100]);
        assert_eq!(std::fs::read(&paths[1]).unwrap(), vec![b'b'; 50]);

        // No part file stays on the disk.
        assert!(!dir.path().join("001 - one.mp3.part").exists());
        assert!(!dir.path().join("002 - two.mp3.part").exists());
    }

    /// The progress must show all the bytes and the state Finished.
    #[tokio::test]
    async fn the_progress_is_complete_after_a_download() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 100]))
            .mount(&server)
            .await;

        let plan = plan_of(&[(1, "10", "one.mp3", 100)]);
        let progress = map();

        fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            progress.clone(),
        )
        .await
        .unwrap();

        let state = state_of(&progress);
        assert_eq!(state.bytes_done, state.bytes_total);
        assert_eq!(state.bytes_total, 100);
        assert_eq!(state.state, DownloadState::Finished);
        assert_eq!(state.percent(), 100);
        assert_eq!(state.file_count, 1);
    }

    /// A part file of 40 bytes gives the header `Range: bytes=40-`. The
    /// function adds the answer to the end of the file.
    #[tokio::test]
    async fn a_part_file_continues_and_keeps_its_bytes() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        let first = vec![b'a'; 40];
        std::fs::write(dir.path().join("001 - one.mp3.part"), &first).unwrap();

        Mock::given(method("GET"))
            .and(path("/api/items/item-1/file/10/download"))
            .and(header("range", "bytes=40-"))
            .respond_with(
                ResponseTemplate::new(206)
                    .insert_header("content-range", "bytes 40-99/100")
                    .set_body_bytes(vec![b'b'; 60]),
            )
            .mount(&server)
            .await;

        let plan = plan_of(&[(1, "10", "one.mp3", 100)]);

        let paths = fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            map(),
        )
        .await
        .unwrap();

        let mut expected = first.clone();
        expected.extend(vec![b'b'; 60]);

        let content = std::fs::read(&paths[0]).unwrap();
        assert_eq!(content.len(), 100);
        assert_eq!(content, expected);
    }

    /// The server does not accept the range and answers 200. The function
    /// writes the file from the start.
    #[tokio::test]
    async fn an_answer_of_200_writes_the_file_from_the_start() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        std::fs::write(dir.path().join("001 - one.mp3.part"), vec![b'x'; 40]).unwrap();

        Mock::given(method("GET"))
            .and(header("range", "bytes=40-"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'c'; 100]))
            .mount(&server)
            .await;

        let plan = plan_of(&[(1, "10", "one.mp3", 100)]);
        let progress = map();

        let paths = fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            progress.clone(),
        )
        .await
        .unwrap();

        assert_eq!(std::fs::read(&paths[0]).unwrap(), vec![b'c'; 100]);
        assert_eq!(state_of(&progress).bytes_done, 100);
    }

    /// A part file that has the full size becomes the complete file. The
    /// function sends no request.
    #[tokio::test]
    async fn a_complete_part_file_gets_the_final_name() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        std::fs::write(dir.path().join("001 - one.mp3.part"), vec![b'd'; 100]).unwrap();

        let plan = plan_of(&[(1, "10", "one.mp3", 100)]);

        let paths = fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            map(),
        )
        .await
        .unwrap();

        assert!(!dir.path().join("001 - one.mp3.part").exists());
        assert_eq!(std::fs::read(&paths[0]).unwrap(), vec![b'd'; 100]);
        assert_eq!(server.received_requests().await.unwrap().len(), 0);
    }

    /// The server sends less than the full file. The function gives an error
    /// and keeps the part file.
    #[tokio::test]
    async fn a_transfer_that_stops_keeps_the_part_file() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'e'; 30]))
            .mount(&server)
            .await;

        let plan = plan_of(&[(1, "10", "one.mp3", 100)]);
        let progress = map();

        let result = fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            progress.clone(),
        )
        .await;

        assert!(result.is_err());

        let part = dir.path().join("001 - one.mp3.part");
        assert!(part.exists(), "the part file must stay on the disk");
        assert_eq!(std::fs::read(&part).unwrap(), vec![b'e'; 30]);
        assert!(!dir.path().join("001 - one.mp3").exists());

        match state_of(&progress).state {
            DownloadState::Failed(_) => {}
            other => panic!("the state must be Failed, but it is {other:?}"),
        }
    }

    /// The second attempt continues the file that the first attempt started.
    #[tokio::test]
    async fn the_second_attempt_completes_the_file() {
        // The two servers give the two answers. The first answer stops in the
        // middle. The second answer completes the file.
        let stops = MockServer::start().await;
        let completes = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();
        let plan = plan_of(&[(1, "10", "one.mp3", 100)]);
        let client = reqwest::Client::new();

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'e'; 30]))
            .mount(&stops)
            .await;

        Mock::given(method("GET"))
            .and(header("range", "bytes=30-"))
            .respond_with(
                ResponseTemplate::new(206)
                    .insert_header("content-range", "bytes 30-99/100")
                    .set_body_bytes(vec![b'f'; 70]),
            )
            .mount(&completes)
            .await;

        let first = fetch_item(&client, &stops.uri(), "secret", &plan, dir.path(), map()).await;
        assert!(first.is_err());

        let paths = fetch_item(
            &client,
            &completes.uri(),
            "secret",
            &plan,
            dir.path(),
            map(),
        )
        .await
        .unwrap();

        let mut expected = vec![b'e'; 30];
        expected.extend(vec![b'f'; 70]);
        assert_eq!(std::fs::read(&paths[0]).unwrap(), expected);
    }

    /// An episode uses the identity of the podcast in the address, and the
    /// identity of the episode in the map of the progress. Two episodes of one
    /// podcast then have two separate bars.
    #[tokio::test]
    async fn an_episode_keeps_the_progress_under_the_episode() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        Mock::given(method("GET"))
            .and(path("/api/items/pod-1/file/700/download"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 20]))
            .mount(&server)
            .await;

        let mut plan = plan_of(&[(1, "700", "one.mp3", 20)]);
        plan.item_id = "pod-1".to_string();
        plan.key = "ep-1".to_string();

        let progress = map();

        fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            progress.clone(),
        )
        .await
        .unwrap();

        let map = progress.read().unwrap();
        assert!(map.get("ep-1").is_some(), "the key must be the episode");
        assert!(
            map.get("pod-1").is_none(),
            "the key must not be the podcast"
        );
        assert_eq!(map.get("ep-1").unwrap().state, DownloadState::Finished);
    }

    /// An answer that is not a success gives an error.
    #[tokio::test]
    async fn an_error_from_the_server_gives_an_error() {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let plan = plan_of(&[(1, "10", "one.mp3", 100)]);

        let result = fetch_item(
            &reqwest::Client::new(),
            &server.uri(),
            "secret",
            &plan,
            dir.path(),
            map(),
        )
        .await;

        assert!(result.is_err());
    }
}
