# Design: offline download that continues and shows progress

Date: 2026-08-09
Status: Approved
Sub-project: 1b (before the API client work)

## 1. Purpose

The offline mode does not play a book. This design corrects the fault. It also
adds two functions that the user asked for: a download that continues after an
interruption, and a progress bar.

## 2. The fault

The application uses `GET /api/items/:id/download`. Measurements on
Audiobookshelf 2.36.0 on 2026-08-09 show this behaviour:

| Property | Measurement |
|---|---|
| Content type | `application/zip`, for every book |
| One audio file in the book | Still `application/zip` |
| `Accept-Ranges` | Absent |

The application writes the archive to the disk. Then it records the path of the
archive in the `downloads` table. Then it gives that path to VLC. VLC cannot
play a ZIP archive. Therefore no downloaded book plays.

## 3. The solution

Do not use the archive endpoint. Get each audio file with its own request.

Measurements of `GET /api/items/:id/file/:ino/download` on the same server:

| Property | Measurement |
|---|---|
| Status with a `Range` header | `206 Partial Content` |
| `Accept-Ranges` | `bytes` |
| `Content-Range` | `bytes 100-1099/2797969` |
| Content type | `audio/mpeg`, the true type of the file |

This gives three results at the same time:

1. The application gets an audio file. Therefore it does not need to open an
   archive. The project does not add a dependency for archives.
2. The `Range` header lets a download continue after an interruption.
3. `Content-Length` gives the size. Therefore the application can show a
   progress bar.

## 4. Data from the server

`GET /api/items/:id` gives `media.audioFiles`. Each element has these fields:

| Field | Use |
|---|---|
| `ino` | The identity of the file in the request path |
| `index` | The sequence of the files. Start at 1. |
| `metadata.size` | The number of bytes. The progress bar uses it. |
| `metadata.filename` | The name of the file on the disk |
| `duration` | The length in seconds |

`media.chapters` gives the chapters. The application keeps the chapters with
the download. Therefore the offline mode does not need to read the chapters
from the audio file.

The field `startOffset` was empty in the test. Therefore the application
calculates the start of each file from the sum of the durations before it.

## 5. Files on the disk

```
~/.local/share/toutui/downloads/<username>/<item_id>/
    001 - <filename>              a complete file
    002 - <filename>.part         a file that is not complete
    item.json                     the metadata, the files, and the chapters
```

The application writes to a file with the name `.part`. When the file is
complete, the application changes the name. Therefore a file without `.part`
is always complete.

The number at the start of the name keeps the sequence of the files.

## 6. How a download continues

For each file:

1. Look at the size of the `.part` file. Call this size `have`.
2. If `have` is equal to the expected size, change the name and continue to the
   next file.
3. If `have` is more than the expected size, delete the file and set `have` to
   0. The file on the server changed.
4. Send the request with the header `Range: bytes=<have>-`.
5. If the answer is `206`, add the bytes to the end of the `.part` file.
6. If the answer is `200`, the server does not support the range. Write the
   file from the start.
7. When the number of bytes is equal to the expected size, change the name.

## 7. Progress

The download task sends its progress to the user interface. The task does not
draw. The user interface reads the progress and draws.

```rust
pub struct DownloadProgress {
    pub item_id:     String,
    pub title:       String,
    pub file_index:  usize,   // the file that the task gets now
    pub file_count:  usize,
    pub bytes_done:  u64,     // all files together
    pub bytes_total: u64,
    pub state:       DownloadState,
}

pub enum DownloadState {
    Running,
    Finished,
    Failed(String),
}
```

The task holds an `Arc<RwLock<HashMap<String, DownloadProgress>>>`. The key is
the item identity. The user interface reads this map on each frame and draws a
`Gauge` for each download that runs.

The task changes `bytes_done` one time for each block of data, and not one time
for each byte. A change for each byte makes too many write operations on the
lock.

## 8. Database

The `downloads` table holds one file path only. A book can have more than one
audio file. Therefore this design adds a table.

```sql
CREATE TABLE IF NOT EXISTS download_files (
    id_item   TEXT    NOT NULL,
    username  TEXT    NOT NULL,
    idx       INTEGER NOT NULL,   -- the sequence, from audioFiles.index
    ino       TEXT    NOT NULL,
    file_path TEXT    NOT NULL,
    size      INTEGER NOT NULL,
    duration  REAL    NOT NULL,
    PRIMARY KEY (id_item, username, idx)
);
```

The column `downloads.file_path` keeps the path of the first audio file.
Therefore the code that plays a book does not change in this sub-project.

This table is migration v3. Sub-project 1 gives the migration runner. This
sub-project uses it.

## 9. What the user sees

- `D` starts the download. The book list shows a bar under the title of the
  book.
- The bar shows the percent, the file number, and the number of megabytes.
- `X` stops a download that runs, and removes the files.
- If the application stops during a download, the `.part` files stay. The next
  `D` on the same book continues from that point.

## 10. Test plan

### 10.1 Unit tests

- The planner calculates the correct sequence from `audioFiles`.
- The planner calculates the correct total number of bytes.
- The resume logic gives `have = 0` for a file that does not exist.
- The resume logic gives the correct start byte for a `.part` file.
- The resume logic deletes a `.part` file that is longer than the expected
  size.
- The progress calculation gives the correct percent.

### 10.2 Integration tests with a mock server

- The client sends `Range: bytes=<have>-` when a `.part` file exists.
- The client adds the bytes to the end of the file, and does not write over it.
- The client accepts a `200` answer and writes the file from the start.
- The client changes the name of the file when the file is complete.
- The client gives an error and keeps the `.part` file when the connection
  stops.

### 10.3 Test with a real server

Get the book `00 Side Jobs`, item `00c4cebd-f893-48d8-8802-5874a4713f22`. This
book has one file of 2 797 969 bytes. Stop the download, then start it again,
and confirm that the file is correct.

## 11. What this design does not do

- It does not change the player. Sub-project 2 replaces VLC.
- It does not download a podcast episode.
- It does not download the cover. A later change can add it.
