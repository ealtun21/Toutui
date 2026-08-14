//! The cover art. See T-23.
//!
//! The module has three parts.
//!
//! 1. **The bytes.** A task asks the server for `GET /api/items/:id/cover` and
//!    writes the answer in a store of the process. The store keeps an item
//!    with no cover, therefore the application asks for such an item one time
//!    only. **The key `R` empties that store** (T-185): the store holds a value
//!    of the server, and a value of the server that the program keeps must go
//!    away with that value. A request that came back with a fault is the
//!    important road: the cover of a book then stayed away for the whole life
//!    of the program, and no key of the user could correct it.
//! 2. **The picture.** The render makes a protocol of `ratatui-image` from the
//!    bytes one time, and it keeps that protocol. The protocol holds the form
//!    of the terminal: the Kitty protocol, Sixel, iTerm2, or blocks of Unicode.
//! 3. **The plan.** `plan_covers` gives the rectangle of each cover. That
//!    function is pure, therefore a test can examine the form of the image and
//!    the narrow terminal with no terminal at all.

use image::DynamicImage;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::FontSize;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::api::client::ApiClient;

/// The smallest width of the whole screen that shows a cover.
///
/// A narrow terminal has no width for a cover and for a text at the same time.
/// The text is more important, therefore the cover goes away.
pub const MIN_WIDTH_FOR_COVER: u16 = 84;

/// The smallest height of the panel that shows a cover.
pub const MIN_HEIGHT_FOR_COVER: u16 = 8;

/// The share of the width that the panel of the covers takes, in percent.
///
/// The user asked for a larger picture on 2026-08-11. The panel took 30 per
/// cent, and a cover of 46 columns then stood in a panel of 34 rows: the
/// picture used two thirds of the height of the panel. See T-50.
const PANEL_PERCENT: u16 = 40;

/// The smallest width of the panel of the covers, in columns.
const PANEL_MIN_WIDTH: u16 = 22;

/// The largest width of the panel of the covers, in columns.
///
/// A very wide terminal must not give the covers the half of the screen.
///
/// The height of the panel gives a second limit, and that limit is the one
/// that binds on most screens: a picture that is as high as the panel needs a
/// number of columns that comes from the height and from the form of the cell.
/// See `width_that_the_height_can_use`.
const PANEL_MAX_WIDTH: u16 = 72;

/// The widest form that a cover has, as the width divided by the height.
///
/// A cover of Audible is square, and a cover of a book is higher than it is
/// wide. A cover that is wider than a square is rare, and the panel gives it
/// the columns of a square. Therefore the value is 1.0.
const WIDEST_COVER: f32 = 1.0;

/// The share of the height that the cover of the media that plays takes, in
/// percent. The rest goes to the covers of the selection.
const PLAYING_PERCENT: u16 = 62;

/// The largest number of covers of a shelf.
pub const SHELF_MAX: usize = 4;

/// The largest size of the answer of the server for one cover.
///
/// A cover of an audiobook is some hundred kilobytes. The limit stops an
/// address that sends bytes with no end. See T-30.
const MAX_COVER_BYTES: u64 = 8 * 1024 * 1024;

/// The largest side of the picture that the application keeps, in pixels.
///
/// The render makes the picture smaller for the area of the screen. That work
/// is faster with a small picture, and a cover of 4000 pixels gives no better
/// result on a terminal.
const MAX_PIXELS: u32 = 640;

/// The largest memory that one picture may take while the program reads it.
const MAX_DECODE_BYTES: u64 = 128 * 1024 * 1024;

/// The largest side of a picture that the program reads, in pixels.
///
/// A small file can name a picture of 60000 by 60000 pixels. The limit stops
/// such a file before the program asks for the memory.
const MAX_DECODE_PIXELS: u32 = 10_000;

/// The condition of one cover in the store of the process.
#[derive(Debug, Clone)]
enum CoverBytes {
    /// A task asks the server now.
    Asked,
    /// The server answered, and it holds no cover of this item. The status of
    /// that answer is 404. The application asks no second time.
    NoCover,
    /// The request came back with a fault. The log holds the reason, and no
    /// view of the program shows it: a cover is a picture beside the text.
    ///
    /// **This is not an item with no cover** (T-185). The old shape of the
    /// store held one condition for both, and the log then said that an item
    /// has no cover for an item whose cover the server holds. The application
    /// asks no second time here too: the render calls `picture` at each frame,
    /// therefore a second request of a fault would be one request of each frame
    /// of each item of the screen. The key `R` empties the store.
    Fault,
    /// The bytes of the picture.
    Ready(Arc<Vec<u8>>),
}

/// The store of the process. It lives outside `App`, therefore a refresh with
/// the key `R` keeps no cover of its own: `forget` empties it.
fn store() -> &'static RwLock<HashMap<String, CoverBytes>> {
    static STORE: OnceLock<RwLock<HashMap<String, CoverBytes>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Forgets every cover of the store. The key `R` calls this.
///
/// **The store holds a value of the server, therefore the key that asks the
/// server again must empty it** — the rule of T-184 for the positions of the
/// live messages, and of T-66 for the shelf of Continue Listening.
///
/// The measurement of 2026-08-14 (T-185): a proxy gave the status 500 to
/// `GET /api/items/:id/cover`, the program wrote that fault in the store, the
/// server answered every request again, and the key `R` then asked the server
/// for seven lists and for **no** cover. The book of the screen kept no picture
/// for the whole life of the program, and the user could correct it in no way.
///
/// A cover that a different client changed on the server comes back at this key
/// for the same reason.
///
/// The pictures of `CoverArt` belong to `App`, and the key `R` makes a new
/// `App`. Therefore the render makes every picture again from the bytes of the
/// new requests.
pub fn forget() {
    if let Ok(mut map) = store().write() {
        map.clear();
    }
}

/// The picker of the process.
///
/// `Picker::from_query_stdio` asks the terminal for the protocol and for the
/// size of the font. That question needs a real terminal. A terminal that does
/// not answer gives `Picker::halfblocks`.
///
/// The application asks one time. A second question during the render would
/// write bytes on the screen of the user.
pub fn picker() -> &'static Picker {
    static PICKER: OnceLock<Picker> = OnceLock::new();
    PICKER.get_or_init(|| {
        if !asks_the_terminal() {
            return Picker::halfblocks();
        }

        Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks())
    })
}

/// Tells if the program may ask the terminal what it can do.
///
/// The question writes bytes to the terminal and it reads the answer. Two
/// conditions make that question a risk, therefore the program does not ask:
///
/// 1. `TOUTUI_NO_COVERS` is set. The user then wants no cover at all, and the
///    program shows no picture.
/// 2. The program runs inside tmux, and `TOUTUI_COVERS_IN_TMUX` is not set.
///    tmux gives the question to the terminal only when the option
///    `allow-passthrough` is on. Without it the answer never comes, and the
///    reader of the crate stays inside `read` and takes the keys of the user.
///
/// Blocks of Unicode need no question, and they draw a cover in every
/// terminal.
fn asks_the_terminal() -> bool {
    if std::env::var_os("TOUTUI_NO_COVERS").is_some() {
        log::info!("[cover] TOUTUI_NO_COVERS is set. The program shows no cover.");
        return false;
    }

    if std::env::var_os("TMUX").is_some() && std::env::var_os("TOUTUI_COVERS_IN_TMUX").is_none() {
        log::info!(
            "[cover] the program runs inside tmux. It asks the terminal \
             nothing, and it draws the covers with blocks of Unicode. Set \
             TOUTUI_COVERS_IN_TMUX=1 to ask."
        );
        return false;
    }

    true
}

/// Tells if the program draws a cover at all.
///
/// `TOUTUI_NO_COVERS` gives the whole width to the text.
pub fn covers_are_on() -> bool {
    std::env::var_os("TOUTUI_NO_COVERS").is_none()
}

/// Asks the server for one cover, if nothing asked for it before.
///
/// The function gives the answer at once and does the work in a task,
/// because the render is not asynchronous.
pub fn request(api: &Arc<ApiClient>, id: &str) {
    if id.is_empty() {
        return;
    }

    {
        let Ok(map) = store().read() else {
            return;
        };
        if map.contains_key(id) {
            return;
        }
    }

    {
        let Ok(mut map) = store().write() else {
            return;
        };
        // A second reader can come between the two locks. The entry decides.
        if map.contains_key(id) {
            return;
        }
        map.insert(id.to_string(), CoverBytes::Asked);
    }

    let api = Arc::clone(api);
    let id = id.to_string();

    tokio::spawn(async move {
        let value = match fetch(&api, &id).await {
            TheAnswer::Bytes(bytes) => {
                log::info!("[cover] the item {} gives {} bytes", id, bytes.len());
                CoverBytes::Ready(Arc::new(bytes))
            }
            TheAnswer::NoCover => {
                log::info!("[cover] the item {} has no cover", id);
                CoverBytes::NoCover
            }
            TheAnswer::Fault(why) => {
                log::info!(
                    // The text of an `ApiError` ends with a full stop already.
                    "[cover] the request of the cover of the item {} came back \
                     with a fault. {} The key R asks the server again.",
                    id,
                    why
                );
                CoverBytes::Fault
            }
        };

        if let Ok(mut map) = store().write() {
            map.insert(id, value);
        }
    });
}

/// What one request of a cover gave.
///
/// **An item with no cover and a request that failed are two different
/// answers** (T-185). The status 404 is the answer of an item with no cover —
/// the rule of T-175, of T-178, and of T-182 — and every other fault is a fault
/// of the request.
enum TheAnswer {
    /// The bytes of the picture.
    Bytes(Vec<u8>),
    /// The server answered, and it holds no cover of this item.
    NoCover,
    /// The request came back with a fault, and this text says why.
    Fault(String),
}

/// Reads the bytes of one cover, with a limit on the size.
async fn fetch(api: &Arc<ApiClient>, id: &str) -> TheAnswer {
    let path = format!("/api/items/{}/cover", id);

    let response = match api
        .send(
            reqwest::Method::GET,
            &path,
            None,
            crate::api::client::Idempotent::Yes,
        )
        .await
    {
        Ok(response) => response,
        // The server answered, and it holds no cover of this item.
        Err(crate::api::client::error::ApiError::NotFound) => return TheAnswer::NoCover,
        Err(error) => return TheAnswer::Fault(format!("{}", error)),
    };

    if let Some(length) = response.content_length() {
        if length > MAX_COVER_BYTES {
            return TheAnswer::Fault(format!(
                // The text stands in a log line beside the text of an
                // `ApiError`, therefore it is a sentence of its own.
                "The cover holds {} bytes, and the limit is {} bytes.",
                length, MAX_COVER_BYTES
            ));
        }
    }

    let mut body: Vec<u8> = Vec::new();
    let mut response = response;

    while let Ok(Some(chunk)) = response.chunk().await {
        if body.len() as u64 + chunk.len() as u64 > MAX_COVER_BYTES {
            return TheAnswer::Fault(format!(
                "The cover sends more than the limit of {} bytes.",
                MAX_COVER_BYTES
            ));
        }
        body.extend_from_slice(&chunk);
    }

    if body.is_empty() {
        return TheAnswer::NoCover;
    }

    TheAnswer::Bytes(body)
}

/// The pictures that the render holds.
///
/// The value belongs to `App`. A refresh with the key `R` makes a new `App`,
/// therefore this map goes away and the render makes the pictures again from
/// the bytes of the store. No request goes to the server a second time.
#[derive(Default)]
pub struct CoverArt {
    pictures: HashMap<String, Option<StatefulProtocol>>,
    /// The width divided by the height of each picture. The plan of the panel
    /// reads it, therefore a cover that is higher than it is wide takes the
    /// whole height of the panel. See T-50.
    forms: HashMap<String, f32>,
}

impl std::fmt::Debug for CoverArt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CoverArt({})",
            crate::ui::keys::counted(self.pictures.len(), "picture")
        )
    }
}

impl CoverArt {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gives the picture of one item, or nothing.
    ///
    /// The function asks the server when no task asked before. It gives
    /// nothing until the answer comes.
    pub fn picture(&mut self, api: &Arc<ApiClient>, id: &str) -> Option<&mut StatefulProtocol> {
        if id.is_empty() {
            return None;
        }

        if !self.pictures.contains_key(id) {
            // The lock must go away before the match. A guard that stands in
            // the expression of a match lives to the end of that match, and
            // `request` then asks for the write lock on the same thread. That
            // stops the application for ever.
            let found = {
                let Ok(map) = store().read() else {
                    return None;
                };
                map.get(id).cloned()
            };

            let bytes = match found {
                Some(CoverBytes::Ready(bytes)) => bytes,
                // The item has no cover, or the request came back with a
                // fault. The program asks no second time, and the key `R`
                // empties the store. See T-185.
                Some(CoverBytes::NoCover) | Some(CoverBytes::Fault) => {
                    self.pictures.insert(id.to_string(), None);
                    return None;
                }
                // The task did not finish. The next frame asks again.
                Some(CoverBytes::Asked) => return None,
                None => {
                    request(api, id);
                    return None;
                }
            };

            let picture = decode(&bytes).map(|image| {
                if image.height() > 0 {
                    let form = image.width() as f32 / image.height() as f32;
                    self.forms.insert(id.to_string(), form);
                }

                picker().new_resize_protocol(image)
            });
            self.pictures.insert(id.to_string(), picture);
        }

        self.pictures.get_mut(id).and_then(|slot| slot.as_mut())
    }

    /// Gives the form of one picture, as the width divided by the height.
    ///
    /// The value comes after the program read the picture. The plan then uses a
    /// square, and the next frame uses the true form. See T-50.
    pub fn form_of(&self, id: &str) -> Option<f32> {
        self.forms.get(id).copied()
    }
}

/// Makes a picture from the bytes of the server.
///
/// The function gives nothing when the bytes are not a picture. A server that
/// sends a broken file must not stop the application.
///
/// The reader has a limit on the memory and a limit on the size in pixels. A
/// small file can name a very large picture, and a reader with no limit then
/// asks for all the memory of the machine.
fn decode(bytes: &[u8]) -> Option<DynamicImage> {
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    limits.max_image_width = Some(MAX_DECODE_PIXELS);
    limits.max_image_height = Some(MAX_DECODE_PIXELS);

    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    reader.limits(limits);
    let image = reader.decode().ok()?;

    if image.width() > MAX_PIXELS || image.height() > MAX_PIXELS {
        return Some(image.thumbnail(MAX_PIXELS, MAX_PIXELS));
    }

    Some(image)
}

/// The rectangle of each cover of the panel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoverPlan {
    /// The cover of the media that plays. It is larger than the covers of the
    /// selection.
    pub playing: Option<Rect>,
    /// The covers of the selection. A series gives more than one, and the
    /// panel then looks like a shelf.
    pub shelf: Vec<Rect>,
}

/// Gives the width of a panel that a picture of the full height fills, in
/// columns.
///
/// A cell of a terminal is higher than it is wide. A picture that is as high as
/// the panel is therefore wider than the number of rows of the panel. A panel
/// that is wider than this value gives the picture no more pixels, and it takes
/// columns of the text for nothing. See T-50.
pub fn width_that_the_height_can_use(height: u16, font: FontSize, ratio: f32) -> u16 {
    if font.width == 0 || font.height == 0 || !ratio.is_finite() || ratio <= 0.0 {
        return PANEL_MAX_WIDTH;
    }

    let pixels = f32::from(height) * f32::from(font.height) * ratio;
    let columns = pixels / f32::from(font.width);

    // The value must stay inside the numbers of a `u16`.
    columns.min(f32::from(u16::MAX)) as u16
}

/// Cuts the main area into the area of the text and the area of the covers.
///
/// The function gives no area for the covers when the screen is too narrow.
/// The text then takes the whole area.
pub fn split_for_covers(area: Rect, screen_width: u16, font: FontSize) -> (Rect, Option<Rect>) {
    if !covers_are_on() {
        return (area, None);
    }

    if screen_width < MIN_WIDTH_FOR_COVER || area.height < MIN_HEIGHT_FOR_COVER {
        return (area, None);
    }

    // A panel that is wider than the height can use gives the picture no more
    // pixels. The form of a cover is not always square, therefore the limit
    // takes the widest form that a cover has. See T-50.
    let of_the_height = width_that_the_height_can_use(area.height, font, WIDEST_COVER);

    let panel_width = (area.width * PANEL_PERCENT / 100)
        .min(of_the_height)
        .clamp(PANEL_MIN_WIDTH, PANEL_MAX_WIDTH);

    // One column stays empty between the text and the covers.
    if area.width < panel_width + PANEL_MIN_WIDTH + 1 {
        return (area, None);
    }

    let text = Rect {
        width: area.width - panel_width - 1,
        ..area
    };

    let panel = Rect {
        x: area.x + area.width - panel_width,
        y: area.y,
        width: panel_width,
        height: area.height,
    };

    (text, Some(panel))
}

/// Gives the largest area inside `slot` that shows a square picture with no
/// change of its form. The area stands in the middle of the slot.
///
/// A cell of a terminal is higher than it is wide. Therefore a square picture
/// needs about two times more columns than rows.
pub fn square_box(slot: Rect, font: FontSize) -> Rect {
    box_of_the_picture(slot, font, 1.0)
}

/// Gives the largest area inside `slot` that shows a picture of the form
/// `ratio` with no change of that form. The area stands in the middle of the
/// slot.
///
/// `ratio` is the width divided by the height of the picture, in pixels. A
/// cover of Audible is square and gives 1.0. A cover of a book is higher than
/// it is wide and gives about 0.66; such a cover then takes the whole height of
/// the slot, and the old rule of the square took two thirds of it. See T-50.
pub fn box_of_the_picture(slot: Rect, font: FontSize, ratio: f32) -> Rect {
    let nothing = Rect {
        width: 0,
        height: 0,
        ..slot
    };

    if slot.width == 0 || slot.height == 0 || font.width == 0 || font.height == 0 {
        return nothing;
    }

    // A form that is not a number, or that is not above zero, gives a square.
    let ratio = if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    };

    // The pixels of the slot.
    let of_the_width = f32::from(slot.width) * f32::from(font.width);
    let of_the_height = f32::from(slot.height) * f32::from(font.height);

    // The picture takes the whole width, or the whole height. The side that
    // gives the smaller picture decides.
    let (pixels_wide, pixels_high) = if of_the_width / ratio <= of_the_height {
        (of_the_width, of_the_width / ratio)
    } else {
        (of_the_height * ratio, of_the_height)
    };

    let width = (pixels_wide / f32::from(font.width)) as u16;
    let height = (pixels_high / f32::from(font.height)) as u16;

    let width = width.min(slot.width);
    let height = height.min(slot.height);

    if width == 0 || height == 0 {
        return nothing;
    }

    Rect {
        x: slot.x + (slot.width - width) / 2,
        y: slot.y + (slot.height - height) / 2,
        width,
        height,
    }
}

/// Gives the rectangle of every cover of the panel.
///
/// `wanted` is the number of covers of the selection. A book gives 1 and a
/// series gives one for each book. The function shows `SHELF_MAX` covers at
/// the most, and fewer when the panel is small.
pub fn plan_covers(
    panel: Rect,
    font: FontSize,
    has_playing: bool,
    wanted: usize,
    form_of_the_large: Option<f32>,
) -> CoverPlan {
    // A picture that the program did not read yet gives no form. The plan then
    // uses a square, and the next frame uses the true form. See T-50.
    let large = form_of_the_large.unwrap_or(1.0);
    let mut plan = CoverPlan::default();

    if panel.width == 0 || panel.height < MIN_HEIGHT_FOR_COVER {
        return plan;
    }

    // The panel holds two covers of a different size only when it is high
    // enough for both. A low panel shows the media that plays and nothing
    // else. The selection also gives no second cover when it is the media
    // that plays, because one cover of one book is enough.
    if has_playing && (wanted == 0 || panel.height < 2 * MIN_HEIGHT_FOR_COVER) {
        plan.playing = Some(box_of_the_picture(panel, font, large));
        return plan;
    }

    let shelf_area = if has_playing {
        let playing_height = (panel.height * PLAYING_PERCENT / 100).max(MIN_HEIGHT_FOR_COVER);

        plan.playing = Some(box_of_the_picture(
            Rect {
                height: playing_height,
                ..panel
            },
            font,
            large,
        ));

        Rect {
            y: panel.y + playing_height,
            height: panel.height - playing_height,
            ..panel
        }
    } else {
        panel
    };

    // One cover of the selection takes the whole area, therefore it is a large
    // cover and it uses the true form. More than one cover goes in a grid of
    // squares: a shelf of a series shows the books, and a small picture is
    // enough there. See T-50.
    plan.shelf = if wanted == 1 && !has_playing {
        shelf_of_one(shelf_area, font, large)
    } else {
        shelf(shelf_area, font, wanted)
    };

    plan
}

/// Puts one cover of the selection in the whole area.
fn shelf_of_one(area: Rect, font: FontSize, ratio: f32) -> Vec<Rect> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let one = box_of_the_picture(area, font, ratio);

    if one.width == 0 || one.height == 0 {
        return Vec::new();
    }

    vec![one]
}

/// Puts the covers of the selection in a grid inside `area`.
fn shelf(area: Rect, font: FontSize, wanted: usize) -> Vec<Rect> {
    let wanted = wanted.min(SHELF_MAX);

    if wanted == 0 || area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    // One cover takes the whole area. More than one goes in two columns,
    // because the panel is higher than it is wide.
    let columns: u16 = if wanted == 1 { 1 } else { 2 };
    let rows = (wanted as u16).div_ceil(columns);

    let cell_width = area.width / columns;
    let cell_height = area.height / rows;

    // A cover of one row or of one column is too small to read. The panel then
    // shows fewer covers.
    if cell_width < 6 || cell_height < 3 {
        if wanted > 1 {
            return shelf(area, font, 1);
        }
        return Vec::new();
    }

    let mut boxes = Vec::with_capacity(wanted);

    for index in 0..wanted as u16 {
        let column = index % columns;
        let row = index / columns;

        let slot = Rect {
            x: area.x + column * cell_width,
            y: area.y + row * cell_height,
            width: cell_width,
            height: cell_height,
        };

        boxes.push(square_box(slot, font));
    }

    boxes
}

#[cfg(test)]
mod tests {
    use super::*;

    const FONT: FontSize = FontSize {
        width: 10,
        height: 20,
    };

    /// The store belongs to the process, therefore every test that touches it
    /// takes this lock. Two such tests would fight for the store.
    fn guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn area(width: u16, height: u16) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    /// A fault of this kind stops the whole application, and no test of a
    /// pure function finds it.
    ///
    /// The first form of `picture` held the read lock of the store during the
    /// whole `match`, because a guard in the expression of a `match` lives to
    /// the end of that `match`. The arm for an unknown item then called
    /// `request`, and `request` asked for the write lock on the same thread.
    /// The application drew one frame and then stopped for ever.
    ///
    /// The test runs the call on its own thread and waits two seconds. A
    /// thread that holds a lock never answers, therefore the test fails.
    #[test]
    fn the_first_ask_for_an_unknown_cover_does_not_stop_the_thread() {
        use std::sync::mpsc;
        use std::time::Duration;

        let _guard = guard();
        forget();

        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            // The address answers nothing. The test examines the locks, and
            // not the server.
            let pool = crate::api::client::endpoint::EndpointPool::new(vec![
                crate::api::client::endpoint::Endpoint::new("http://127.0.0.1:1", 0),
            ]);
            let api =
                Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).expect("a client"));

            // `request` starts a task, therefore the thread needs a runtime.
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("a runtime");
            let _guard = runtime.enter();

            let mut art = CoverArt::new();
            let first = art.picture(&api, "an-item-with-no-answer").is_none();
            let second = art.picture(&api, "an-item-with-no-answer").is_none();

            let _ = sender.send(first && second);
        });

        let answer = receiver.recv_timeout(Duration::from_secs(2));
        assert_eq!(
            answer,
            Ok(true),
            "the ask for a cover stopped the thread, or it gave a picture"
        );
    }

    /// **The key `R` asks the server for every list again, therefore it must
    /// ask for every cover again too.** See T-185.
    ///
    /// The measurement of 2026-08-14: a proxy gave the status 500 to
    /// `GET /api/items/:id/cover`, the program wrote that fault in the store,
    /// the server answered every request again, and the key `R` then asked the
    /// server for seven lists and for no cover at all. The book of the screen
    /// kept no picture for the whole life of the program.
    ///
    /// The store belongs to the process, therefore the parts of this test stay
    /// in one function.
    #[test]
    fn the_key_r_empties_the_store_of_the_covers() {
        let _guard = guard();
        forget();

        {
            let mut map = store().write().expect("the store of the covers");
            map.insert(
                "a book".to_string(),
                CoverBytes::Ready(Arc::new(vec![1, 2, 3])),
            );
            map.insert("a book of a fault".to_string(), CoverBytes::Fault);
            map.insert("a book with no cover".to_string(), CoverBytes::NoCover);
        }

        assert_eq!(store().read().expect("the store of the covers").len(), 3);

        forget();

        assert!(
            store().read().expect("the store of the covers").is_empty(),
            "the key R must empty the store: a cover that came back with a \
             fault stays there for the whole life of the program, and no other \
             key of the user corrects it"
        );
    }

    /// A server of one status, on an address of this machine. The test needs no
    /// network and no sandbox.
    fn a_server_of_one_status(status: &'static str) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a port of the machine");
        let address = format!("http://{}", listener.local_addr().expect("the address"));

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { break };
                use std::io::{Read, Write};

                let mut buffer = [0u8; 2048];
                let _ = stream.read(&mut buffer);
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        status
                    )
                    .as_bytes(),
                );
            }
        });

        address
    }

    /// **An item with no cover and a request that failed are two different
    /// answers.** See T-185.
    ///
    /// The old shape of `fetch` gave nothing for both, therefore the log said
    /// that an item has no cover for an item whose cover the server holds. The
    /// status 404 is the answer of an item with no cover — the rule of T-175,
    /// of T-178, and of T-182.
    #[test]
    fn the_status_404_is_an_item_with_no_cover_and_a_fault_is_a_fault() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime");

        let client_of = |address: &str| {
            let pool = crate::api::client::endpoint::EndpointPool::new(vec![
                crate::api::client::endpoint::Endpoint::new(address, 0),
            ]);
            Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).expect("a client"))
        };

        let of_404 = client_of(&a_server_of_one_status("404 Not Found"));
        assert!(
            matches!(
                runtime.block_on(fetch(&of_404, "a book")),
                TheAnswer::NoCover
            ),
            "the status 404 is the answer of an item with no cover"
        );

        let of_500 = client_of(&a_server_of_one_status("500 Internal Server Error"));
        match runtime.block_on(fetch(&of_500, "a book")) {
            TheAnswer::Fault(why) => assert!(!why.is_empty(), "the fault must say why"),
            _ => panic!("the status 500 is a fault of the request, and not an item with no cover"),
        }
    }

    /// Makes a PNG file that names a picture of a given size, and that holds
    /// no picture at all. A reader with no limit asks for the memory of the
    /// whole picture.
    fn png_header(width: u32, height: u32) -> Vec<u8> {
        fn crc32(data: &[u8]) -> u32 {
            let mut crc: u32 = 0xffff_ffff;
            for byte in data {
                crc ^= u32::from(*byte);
                for _ in 0..8 {
                    let mask = (crc & 1).wrapping_neg();
                    crc = (crc >> 1) ^ (0xedb8_8320 & mask);
                }
            }
            !crc
        }

        let mut chunk = b"IHDR".to_vec();
        chunk.extend_from_slice(&width.to_be_bytes());
        chunk.extend_from_slice(&height.to_be_bytes());
        // Eight bits for each colour, the type "true colour", and no filter.
        chunk.extend_from_slice(&[8, 2, 0, 0, 0]);

        let mut file = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        file.extend_from_slice(&13u32.to_be_bytes());
        file.extend_from_slice(&chunk);
        file.extend_from_slice(&crc32(&chunk).to_be_bytes());
        file
    }

    #[test]
    fn a_file_that_is_not_a_picture_gives_no_picture() {
        assert!(decode(b"this is not a picture").is_none());
        assert!(decode(&[]).is_none());
    }

    #[test]
    fn a_picture_that_is_too_large_gives_no_picture() {
        // 60000 by 60000 pixels of three bytes is 10.8 gigabytes. The limit
        // must stop the reader before it asks for that memory.
        assert!(decode(&png_header(60_000, 60_000)).is_none());
    }

    #[test]
    fn a_large_picture_becomes_small() {
        use image::ImageFormat;

        let large = DynamicImage::new_rgb8(1500, 1000);
        let mut bytes: Vec<u8> = Vec::new();
        large
            .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
            .expect("a PNG file");

        let picture = decode(&bytes).expect("a picture");
        assert!(picture.width() <= MAX_PIXELS);
        assert!(picture.height() <= MAX_PIXELS);
        // The form of the picture stays the same. A whole number of pixels
        // gives a small difference, therefore the test allows one part in a
        // hundred.
        let before = 1500.0 / 1000.0;
        let after = f64::from(picture.width()) / f64::from(picture.height());
        assert!(
            (before - after).abs() < 0.01,
            "the form changed from {} to {}",
            before,
            after
        );
    }

    /// The tests of the panel need the covers to be on. The variable belongs
    /// to the process, therefore the test sets it and gives it back.
    struct WithCovers(Option<std::ffi::OsString>);

    impl WithCovers {
        fn on() -> Self {
            let before = std::env::var_os("TOUTUI_NO_COVERS");
            std::env::remove_var("TOUTUI_NO_COVERS");
            WithCovers(before)
        }
    }

    impl Drop for WithCovers {
        fn drop(&mut self) {
            match self.0.take() {
                Some(value) => std::env::set_var("TOUTUI_NO_COVERS", value),
                None => std::env::remove_var("TOUTUI_NO_COVERS"),
            }
        }
    }

    #[test]
    fn a_narrow_screen_gives_the_whole_area_to_the_text() {
        let _covers = WithCovers::on();
        let main = area(80, 20);
        let (text, panel) = split_for_covers(main, 80, FONT);
        assert_eq!(text, main);
        assert_eq!(panel, None);
    }

    #[test]
    fn a_screen_of_the_smallest_width_shows_a_cover() {
        let _covers = WithCovers::on();
        let main = area(MIN_WIDTH_FOR_COVER, 20);
        let (_text, panel) = split_for_covers(main, MIN_WIDTH_FOR_COVER, FONT);
        assert!(panel.is_some(), "the smallest width must show a cover");
    }

    #[test]
    fn a_low_panel_gives_the_whole_area_to_the_text() {
        let _covers = WithCovers::on();
        let main = area(150, MIN_HEIGHT_FOR_COVER - 1);
        let (text, panel) = split_for_covers(main, 150, FONT);
        assert_eq!(text, main);
        assert_eq!(panel, None);
    }

    #[test]
    fn the_panel_stands_at_the_right_and_leaves_a_column() {
        let _covers = WithCovers::on();
        let main = area(150, 25);
        let (text, panel) = split_for_covers(main, 150, FONT);
        let panel = panel.expect("a wide screen shows a cover");

        assert_eq!(panel.x + panel.width, main.x + main.width);
        assert_eq!(text.x, main.x);
        assert_eq!(text.width + 1 + panel.width, main.width);
    }

    #[test]
    fn the_panel_is_never_wider_than_the_limit() {
        let _covers = WithCovers::on();
        let (_text, panel) = split_for_covers(area(400, 40), 400, FONT);
        assert_eq!(panel.expect("a cover").width, PANEL_MAX_WIDTH);
    }

    #[test]
    fn the_panel_is_generous() {
        let _covers = WithCovers::on();
        // A screen of 150 columns must give the cover more than 30 columns.
        let (_text, panel) = split_for_covers(area(150, 29), 150, FONT);
        assert!(panel.expect("a cover").width >= 30);
    }

    /// A cover of a book is higher than it is wide. The old rule of the square
    /// then took two thirds of the height of the slot. See T-50.
    #[test]
    fn a_picture_that_is_high_takes_the_whole_height() {
        // The cell is 10 by 20 pixels. A slot of 40 by 20 cells holds 400 by
        // 400 pixels.
        let slot = area(40, 20);

        // A square picture takes the 400 pixels of the width: 40 columns and
        // 20 rows.
        let square = box_of_the_picture(slot, FONT, 1.0);
        assert_eq!((square.width, square.height), (40, 20));

        // A picture of two thirds takes the whole height, and it needs 26
        // columns of the 40.
        let high = box_of_the_picture(slot, FONT, 2.0 / 3.0);
        assert_eq!(high.height, 20, "the picture must take every row");
        assert_eq!(high.width, 26);
        // The picture stands in the middle of the slot.
        assert_eq!(high.x, slot.x + (40 - 26) / 2);
    }

    /// A form that the program cannot use gives a square, and it stops nothing.
    #[test]
    fn a_form_that_is_not_a_number_gives_a_square() {
        let slot = area(40, 20);
        let square = box_of_the_picture(slot, FONT, 1.0);

        assert_eq!(box_of_the_picture(slot, FONT, f32::NAN), square);
        assert_eq!(box_of_the_picture(slot, FONT, 0.0), square);
        assert_eq!(box_of_the_picture(slot, FONT, -2.0), square);
        assert_eq!(box_of_the_picture(slot, FONT, f32::INFINITY), square);
    }

    /// The panel must not be wider than the height of the panel can use. Those
    /// columns give the picture no pixel, and they take the width of the text.
    #[test]
    fn the_panel_uses_the_height_of_the_screen() {
        let _covers = WithCovers::on();

        // A screen of 160 columns and a main area of 34 rows. The cell is 10 by
        // 20 pixels, therefore a square picture of 34 rows needs 68 columns.
        assert_eq!(width_that_the_height_can_use(34, FONT, 1.0), 68);

        let (_text, panel) = split_for_covers(area(160, 34), 160, FONT);
        let panel = panel.expect("a wide screen shows a cover");

        // 40 per cent of 160 is 64, and the height can use 68. Therefore the
        // panel is 64 columns wide, and the old rule gave 46.
        assert_eq!(panel.width, 64);

        // A low panel takes fewer columns, because more columns would give the
        // picture no pixel.
        let (_text, low) = split_for_covers(area(160, 12), 160, FONT);
        assert_eq!(low.expect("a cover").width, 24);
    }

    /// The cover of one book fills the panel now. A shelf of a series keeps the
    /// small covers. See T-50.
    #[test]
    fn one_cover_fills_the_panel_and_a_shelf_does_not() {
        let panel = area(64, 34);

        let one = plan_covers(panel, FONT, false, 1, Some(2.0 / 3.0));
        let box_of_one = one.shelf[0];
        assert_eq!(box_of_one.height, 34, "one cover takes every row");

        // Four covers of a series stay small.
        let four = plan_covers(panel, FONT, false, 4, Some(2.0 / 3.0));
        assert_eq!(four.shelf.len(), 4);

        for small in &four.shelf {
            assert!(
                small.height < box_of_one.height,
                "a cover of a shelf must stay small"
            );
        }
    }

    #[test]
    fn a_square_box_keeps_the_form_of_the_picture() {
        // The cell is 10 by 20 pixels. Therefore a square needs two times more
        // columns than rows.
        let result = square_box(area(40, 10), FONT);
        assert_eq!(result.width, 20);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn a_square_box_that_is_wide_takes_the_height() {
        let result = square_box(area(100, 5), FONT);
        assert_eq!(result.height, 5);
        assert_eq!(result.width, 10);
    }

    #[test]
    fn a_square_box_stands_in_the_middle() {
        let result = square_box(area(40, 5), FONT);
        assert_eq!(result.width, 10);
        assert_eq!(result.height, 5);
        assert_eq!(result.x, 15);
    }

    #[test]
    fn a_square_box_of_no_size_gives_no_size() {
        assert_eq!(square_box(area(0, 10), FONT).width, 0);
        assert_eq!(square_box(area(10, 0), FONT).height, 0);
    }

    #[test]
    fn one_book_gives_one_cover() {
        let plan = plan_covers(area(40, 24), FONT, false, 1, None);
        assert_eq!(plan.playing, None);
        assert_eq!(plan.shelf.len(), 1);
    }

    #[test]
    fn a_series_gives_a_shelf() {
        let plan = plan_covers(area(40, 24), FONT, false, 3, None);
        assert_eq!(plan.shelf.len(), 3);
    }

    #[test]
    fn a_shelf_never_shows_more_than_the_limit() {
        let plan = plan_covers(area(40, 24), FONT, false, 20, None);
        assert_eq!(plan.shelf.len(), SHELF_MAX);
    }

    #[test]
    fn no_cover_of_a_shelf_covers_another_cover() {
        let plan = plan_covers(area(40, 24), FONT, true, 4, None);

        let mut boxes = plan.shelf.clone();
        if let Some(playing) = plan.playing {
            boxes.push(playing);
        }

        for (index, first) in boxes.iter().enumerate() {
            for second in boxes.iter().skip(index + 1) {
                assert!(
                    !first.intersects(*second),
                    "the cover {:?} covers {:?}",
                    first,
                    second
                );
            }
        }
    }

    #[test]
    fn every_cover_stays_inside_the_panel() {
        let panel = Rect {
            x: 100,
            y: 5,
            width: 40,
            height: 24,
        };

        for wanted in 1..=SHELF_MAX {
            for has_playing in [false, true] {
                let plan = plan_covers(panel, FONT, has_playing, wanted, None);
                let mut boxes = plan.shelf.clone();
                if let Some(playing) = plan.playing {
                    boxes.push(playing);
                }

                for one in boxes {
                    assert!(
                        panel.union(one) == panel,
                        "the cover {:?} left the panel {:?}",
                        one,
                        panel
                    );
                }
            }
        }
    }

    #[test]
    fn the_cover_of_the_media_that_plays_is_larger() {
        let area_of = |rect: &Rect| u32::from(rect.width) * u32::from(rect.height);

        for height in (2 * MIN_HEIGHT_FOR_COVER)..40 {
            for wanted in 1..=SHELF_MAX {
                let plan = plan_covers(area(40, height), FONT, true, wanted, None);
                let playing = plan.playing.expect("the media that plays has a cover");

                for selected in &plan.shelf {
                    assert!(
                        area_of(&playing) > area_of(selected),
                        "at {} rows the cover that plays {:?} is not larger than {:?}",
                        height,
                        playing,
                        selected
                    );
                }
            }
        }
    }

    #[test]
    fn a_low_panel_shows_the_media_that_plays_only() {
        let plan = plan_covers(area(40, MIN_HEIGHT_FOR_COVER), FONT, true, 3, None);
        assert!(plan.playing.is_some());
        assert!(plan.shelf.is_empty());
    }

    #[test]
    fn the_selection_that_plays_gives_one_large_cover() {
        // The caller asks for no cover of the selection when the selection is
        // the media that plays. The panel then holds one cover only, and that
        // cover takes the whole panel.
        let panel = area(40, 24);
        let plan = plan_covers(panel, FONT, true, 0, None);
        let playing = plan.playing.expect("the media that plays has a cover");

        assert!(plan.shelf.is_empty());
        assert_eq!(playing, square_box(panel, FONT));
    }

    #[test]
    fn a_small_panel_shows_one_cover_and_not_four() {
        // The panel is high enough for one cover, and each cover of a grid of
        // four would be three columns wide. The panel then shows one.
        let plan = plan_covers(area(10, 10), FONT, false, 4, None);
        assert_eq!(plan.shelf.len(), 1);
    }

    #[test]
    fn a_panel_that_is_too_low_shows_no_cover() {
        let plan = plan_covers(area(40, MIN_HEIGHT_FOR_COVER - 1), FONT, false, 3, None);
        assert!(plan.playing.is_none());
        assert!(plan.shelf.is_empty());
    }
}
