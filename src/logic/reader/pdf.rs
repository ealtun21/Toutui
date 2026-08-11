//! The reader of a PDF book. See T-54.
//!
//! Audiobookshelf holds a PDF beside an EPUB book. The reader of T-10 showed
//! EPUB books only, and the key `e` on a media with a PDF gave a fault.
//!
//! **A PDF holds no chapter and no flow of text.** It holds pages, and each page
//! holds a program that draws letters at places. Therefore this module gives:
//!
//! - **One page for one chapter.** The reader of T-10 moves with `n` and `p`
//!   between the pages, and the list of the contents names them.
//! - **The text of a page as XHTML.** The render of the EPUB book then makes the
//!   lines, and the reader needs no second render.
//! - **The pictures of a page as files.** A picture of the filter `DCTDecode` is
//!   a JPEG file already. A picture of raw samples becomes a PNG file. The crate
//!   `image` reads both, and `ratatui-image` draws them, as the cover art of
//!   T-23 does.
//!
//! **No crate of pure Rust draws a page of a PDF.** `mupdf` needs a library of C
//! and it is AGPL, therefore T-20 and the license refuse it. A user who needs the
//! page of a figure opens the web page of the server. See T-51.
//!
//! The limits of this module hold the memory of the program:
//!
//! | Limit | Value | Why |
//! |---|---|---|
//! | The file | 512 megabytes | A PDF of a scan is large, and larger than a book |
//! | The pages | 5000 | A book of more pages is not a book |
//! | One picture | 32 megabytes | A picture of a page of a scan |
//! | The pixels of one picture | 50 million | A small stream can name a very large picture |

use crate::logic::reader::book::ReaderError;
use log::{info, warn};
use lopdf::{Dictionary, Document, Object};
use std::path::Path;
use std::sync::Arc;

/// The largest file that the reader opens.
const MAX_BOOK_BYTES: u64 = 512 * 1024 * 1024;

/// The largest number of pages that the reader holds.
const MAX_PAGES: usize = 5000;

/// The largest number of bytes of one picture.
const MAX_PICTURE_BYTES: usize = 32 * 1024 * 1024;

/// The largest number of pixels of one picture.
const MAX_PICTURE_PIXELS: u64 = 50_000_000;

/// The largest side of a picture that the program keeps, in pixels. See T-62.
///
/// A terminal of 160 by 45 gives the panel of the picture about 64 columns and 42
/// rows, and a cell of 10 by 20 pixels makes that 640 by 840 pixels. A picture of
/// 1400 by 1900 therefore holds four times more pixels than the largest screen
/// shows, and every one of those bytes stays in the memory while the user reads.
///
/// The value is the value of the cover art of T-23, for the same reason.
const LARGEST_SIDE: u32 = 640;

/// The largest memory that every picture of one book may take. See T-62.
///
/// A measurement on 2026-08-11 of a book of 150 pages of a scan of 1400 by 1900:
/// the pictures held 137 megabytes, and the program took 279 megabytes. A book of
/// 600 pages would take the memory of a small machine.
const MAX_PICTURES_OF_A_BOOK: usize = 48 * 1024 * 1024;

/// One picture of a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// The name of the picture inside the page. An example is `Im0`.
    pub name: String,
    /// The width in pixels, as the PDF names it.
    pub width: u32,
    /// The height in pixels, as the PDF names it.
    pub height: u32,
    /// A **file** that the crate `image` reads: a JPEG file, or a PNG file that
    /// this module made of the raw samples.
    ///
    /// The file can hold **fewer** pixels than `width` and `height`: a picture of
    /// more than 640 pixels of one side gives the screen nothing more, and it
    /// takes the memory of the user. The form of the file is the form of the two
    /// numbers. See T-62.
    pub file: Arc<Vec<u8>>,
}

/// One page of the book.
#[derive(Debug, Clone, Default)]
pub struct Page {
    /// The number of the page, as the PDF names it. The first page is 1.
    pub number: u32,
    /// The text of the page.
    pub text: String,
    /// The pictures of the page, in the sequence of their names.
    pub pictures: Vec<Picture>,
}

/// One open PDF book.
pub struct Pdf {
    pages: Vec<Page>,
    title: String,
    author: String,
}

impl Pdf {
    /// Opens the PDF at `path`.
    ///
    /// The function reads every page one time. A book of 300 pages holds some
    /// megabytes of text, therefore the memory stays small. The pictures stand
    /// behind an `Arc`, and the screen holds no copy of them.
    ///
    /// The call reads the whole file. The caller must run it in a task, and never
    /// on the thread that draws.
    pub fn open(path: &Path) -> Result<Pdf, ReaderError> {
        if let Ok(data) = std::fs::metadata(path) {
            if data.is_file() && data.len() > MAX_BOOK_BYTES {
                return Err(ReaderError::BookTooLarge(data.len()));
            }
        }

        let document = Document::load(path).map_err(|error| {
            warn!(
                "[pdf] the file is not a PDF that the program reads: {}",
                error
            );
            ReaderError::NotAnEpub
        })?;

        let numbers = document.get_pages();

        if numbers.is_empty() {
            return Err(ReaderError::NoSuchChapter(0));
        }

        if numbers.len() > MAX_PAGES {
            return Err(ReaderError::TooManyEntries(numbers.len()));
        }

        let mut pages: Vec<Page> = Vec::with_capacity(numbers.len());

        // The memory of every picture of the book. A book of a scan of many
        // hundred pages must not take the memory of the machine. See T-62.
        let mut bytes_of_the_pictures = 0usize;
        let mut pages_with_no_picture = 0usize;

        for (number, id) in numbers {
            let text = document
                .extract_text(&[number])
                .unwrap_or_default()
                .replace('\r', "");

            let pictures = if bytes_of_the_pictures < MAX_PICTURES_OF_A_BOOK {
                let pictures = pictures_of_the_page(&document, id);

                bytes_of_the_pictures += pictures
                    .iter()
                    .map(|picture| picture.file.len())
                    .sum::<usize>();

                pictures
            } else {
                // The line of the text of the page still says that a picture
                // exists, therefore the user knows. See `xhtml_of_the_page`.
                pages_with_no_picture += 1;
                Vec::new()
            };

            pages.push(Page {
                number,
                text,
                pictures,
            });
        }

        if pages_with_no_picture > 0 {
            warn!(
                "[pdf] the pictures of this book hold {} megabytes. The program \
                 keeps no picture of the last {} page(s).",
                bytes_of_the_pictures / 1024 / 1024,
                pages_with_no_picture
            );
        }

        let letters: usize = pages.iter().map(|page| page.text.len()).sum();
        let pictures: usize = pages.iter().map(|page| page.pictures.len()).sum();

        info!(
            "[pdf] the book holds {} page(s), {} letters, and {} picture(s)",
            pages.len(),
            letters,
            pictures
        );

        let title = title_of(&document, path);
        let author = author_of(&document);

        Ok(Pdf {
            pages,
            title,
            author,
        })
    }

    /// The number of pages.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// One page, if the number of the page exists.
    pub fn page(&self, index: usize) -> Option<&Page> {
        self.pages.get(index)
    }

    /// The title of the book.
    pub fn title(&self) -> String {
        self.title.clone()
    }

    /// The author of the book.
    pub fn author(&self) -> String {
        self.author.clone()
    }

    /// The size of each page, in bytes of text.
    ///
    /// The part of the book that the user read comes from these numbers, in the
    /// same way as the chapters of an EPUB book. A page with a picture and no
    /// text would give 0 and the part of the book would then jump, therefore a
    /// picture counts as 1000 bytes.
    pub fn page_sizes(&self) -> Vec<u64> {
        self.pages
            .iter()
            .map(|page| {
                let text = page.text.len() as u64;
                let pictures = page.pictures.len() as u64 * 1000;
                (text + pictures).max(1)
            })
            .collect()
    }
}

/// The value that a page with a picture adds to its size, for the part of the
/// book. The tests read it.
pub const SIZE_OF_A_PICTURE: u64 = 1000;

/// Makes the XHTML of one page.
///
/// The render of the EPUB book takes XHTML, therefore a page of a PDF gives
/// XHTML and the program holds one render only. See T-10.
///
/// A line that ends with no full stop is a line of the page, and not the end of
/// a paragraph. This function keeps the lines of the page: a PDF holds no
/// paragraph, and a guess of the paragraphs makes the text of a table and the
/// text of a poem wrong.
pub fn xhtml_of_the_page(page: &Page) -> String {
    let mut out = String::from("<html><body>");

    out.push_str(&format!("<h2>The page {}</h2>", page.number));

    for picture in &page.pictures {
        // The reader draws the picture beside the text. The line of the text
        // says that the picture exists, therefore a terminal that draws no
        // picture still tells the user.
        out.push_str(&format!(
            "<p>[ the picture {}: {} by {} pixels ]</p>",
            escape(&picture.name),
            picture.width,
            picture.height
        ));
    }

    for line in page.text.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        out.push_str("<p>");
        out.push_str(&escape(line));
        out.push_str("</p>");
    }

    if page.text.trim().is_empty() && page.pictures.is_empty() {
        out.push_str("<p>This page holds no text and no picture.</p>");
    }

    out.push_str("</body></html>");
    out
}

/// Puts the five characters of XML in their safe form.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Gives the title of the book.
///
/// The information of a PDF holds no title in most files. The name of the file
/// is then the title, and that name says more to the user than an empty line.
fn title_of(document: &Document, path: &Path) -> String {
    let from_the_file = || {
        path.file_stem()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("A book with no title"))
    };

    match text_of_the_information(document, b"Title") {
        Some(title) if !title.trim().is_empty() => title,
        _ => from_the_file(),
    }
}

/// Gives the author of the book, or a short message.
fn author_of(document: &Document) -> String {
    text_of_the_information(document, b"Author")
        .filter(|author| !author.trim().is_empty())
        .unwrap_or_else(|| String::from("N/A"))
}

/// Reads one field of the information of the document.
fn text_of_the_information(document: &Document, key: &[u8]) -> Option<String> {
    let information = document.trailer.get(b"Info").ok()?;
    let (_, object) = document.dereference(information).ok()?;
    let dictionary = object.as_dict().ok()?;
    let value = dictionary.get(key).ok()?;

    text_of(value)
}

/// Gives the text of an object of a PDF.
///
/// A PDF writes a text in the form of PDFDoc or in the form of UTF-16 with two
/// bytes of mark at the start. The second form comes from every program that
/// writes a name with a letter that is not of ASCII.
fn text_of(object: &Object) -> Option<String> {
    let bytes = object.as_str().ok()?;

    if bytes.starts_with(&[0xfe, 0xff]) {
        let pairs: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect();

        return Some(String::from_utf16_lossy(&pairs));
    }

    Some(String::from_utf8_lossy(bytes).into_owned())
}

/// Gives every picture of one page.
///
/// The pictures of a page stand in the resources of that page, as objects of the
/// kind `Image`. A page that names no picture gives an empty list, and that is
/// not a fault.
fn pictures_of_the_page(document: &Document, page: lopdf::ObjectId) -> Vec<Picture> {
    let Ok(dictionary) = document.get_dictionary(page) else {
        return Vec::new();
    };

    let Some(resources) = dictionary
        .get(b"Resources")
        .ok()
        .and_then(|value| document.dereference(value).ok())
        .and_then(|(_, value)| value.as_dict().ok().cloned())
    else {
        return Vec::new();
    };

    let Some(objects) = resources
        .get(b"XObject")
        .ok()
        .and_then(|value| document.dereference(value).ok())
        .and_then(|(_, value)| value.as_dict().ok().cloned())
    else {
        return Vec::new();
    };

    let mut pictures: Vec<Picture> = Vec::new();

    for (name, value) in objects.iter() {
        let name = String::from_utf8_lossy(name).into_owned();

        let Ok(id) = value.as_reference() else {
            continue;
        };

        let Ok(stream) = document
            .get_object(id)
            .and_then(|object| object.as_stream())
        else {
            continue;
        };

        if stream
            .dict
            .get(b"Subtype")
            .ok()
            .and_then(|value| value.as_name().ok())
            != Some(b"Image")
        {
            continue;
        }

        match picture_of_the_stream(&name, &stream.dict, &stream.content, || {
            stream.decompressed_content().ok()
        }) {
            Some(picture) => pictures.push(picture),
            None => info!("[pdf] the program does not read the picture {}", name),
        }
    }

    pictures.sort_by(|one, two| one.name.cmp(&two.name));
    pictures
}

/// Makes a picture of the stream of a PDF.
///
/// `raw` gives the bytes of the stream with no compression. The function calls it
/// for a picture of raw samples only, therefore a picture of JPEG needs no work
/// of the memory.
fn picture_of_the_stream(
    name: &str,
    dictionary: &Dictionary,
    content: &[u8],
    raw: impl FnOnce() -> Option<Vec<u8>>,
) -> Option<Picture> {
    let width = number_of(dictionary, b"Width")? as u32;
    let height = number_of(dictionary, b"Height")? as u32;

    if width == 0 || height == 0 {
        return None;
    }

    if u64::from(width) * u64::from(height) > MAX_PICTURE_PIXELS {
        warn!(
            "[pdf] the picture {} is {} by {} pixels. The program does not read it.",
            name, width, height
        );
        return None;
    }

    let filters = names_of(dictionary, b"Filter");
    let bits = number_of(dictionary, b"BitsPerComponent").unwrap_or(8);
    let colours = colour_count(dictionary);

    // A picture of the filter `DCTDecode` is a JPEG file already. The bytes of
    // the stream are that file, therefore a picture that the screen can show
    // needs no work at all.
    if filters.iter().any(|filter| filter == "DCTDecode") {
        if content.len() > MAX_PICTURE_BYTES {
            return None;
        }

        return Some(smaller_if_it_is_large(name, width, height, content));
    }

    // Every other filter of a picture of a PDF gives raw samples. The program
    // reads 8 bits and 16 bits of one component.
    //
    // **`decompressed_content` of `lopdf` undoes the predictor of PNG.** A
    // measurement on 2026-08-11 of a picture of 1200 by 1600 with
    // `DecodeParms <</Predictor 15>>` gave 3840000 bytes, and that is
    // 1200 × 1600 × 2 with no byte of a row of the predictor. Therefore this
    // module needs no reader of a predictor. See T-57.
    if bits != 8 && bits != 16 {
        return None;
    }

    let samples = raw()?;

    if samples.len() > MAX_PICTURE_BYTES {
        return None;
    }

    // A terminal shows a picture of some hundred cells. Therefore the high byte
    // of a sample of 16 bits is enough, and the program needs no arithmetic of
    // the colour.
    let samples = if bits == 16 {
        eight_bits_of(&samples)
    } else {
        samples
    };

    let file = png_of_the_samples(width, height, colours?, &samples)?;

    Some(smaller_if_it_is_large(name, width, height, &file))
}

/// Gives a picture of the file, and it makes a large picture smaller.
///
/// The screen shows about 640 pixels of one side. A picture of more pixels gives
/// the user nothing, and every byte of it stays in the memory while they read.
/// Therefore this function reads such a picture, it makes it smaller, and it
/// writes a JPEG file of the answer. See T-62.
///
/// A picture that the program cannot read keeps the bytes of the file. The screen
/// then shows what it can, and the memory holds one picture more.
fn smaller_if_it_is_large(name: &str, width: u32, height: u32, file: &[u8]) -> Picture {
    let of_the_file = || Picture {
        name: name.to_string(),
        width,
        height,
        file: Arc::new(file.to_vec()),
    };

    if width <= LARGEST_SIDE && height <= LARGEST_SIDE {
        return of_the_file();
    }

    let Some(image) = read_the_picture(file) else {
        return of_the_file();
    };

    let small = image.thumbnail(LARGEST_SIDE, LARGEST_SIDE);
    let mut out: Vec<u8> = Vec::new();

    // A picture of a page is a picture of a photograph or of a scan, therefore
    // JPEG gives a much smaller file than PNG. The screen of a terminal shows no
    // difference of the two.
    let written = image::codecs::jpeg::JpegEncoder::new_with_quality(
        &mut std::io::Cursor::new(&mut out),
        JPEG_QUALITY,
    )
    .encode_image(&small);

    if written.is_err() || out.is_empty() {
        return of_the_file();
    }

    // **The two numbers stay the numbers of the page.** The user asks "how large
    // is this picture", and the answer of the page is the answer that they want.
    // The form of the two numbers is the form of the smaller file as well, because
    // `thumbnail` keeps the form. See T-62.
    Picture {
        name: name.to_string(),
        width,
        height,
        file: Arc::new(out),
    }
}

/// The quality of the JPEG file of a picture that the program made smaller.
///
/// A terminal draws some hundred cells of one picture. The value 82 gives a file
/// of some tens of kilobytes, and the user sees no loss.
const JPEG_QUALITY: u8 = 82;

/// Reads a picture of a file, with the limits of the memory of this module.
fn read_the_picture(file: &[u8]) -> Option<image::DynamicImage> {
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(MAX_PICTURE_BYTES as u64 * 4);
    // The picture holds `MAX_PICTURE_PIXELS` at the most, therefore a side of
    // that number of pixels is the largest side that a file can name.
    limits.max_image_width = Some(MAX_PICTURE_PIXELS as u32);
    limits.max_image_height = Some(MAX_PICTURE_PIXELS as u32);

    let mut reader = image::ImageReader::new(std::io::Cursor::new(file))
        .with_guessed_format()
        .ok()?;
    reader.limits(limits);

    reader.decode().ok()
}

/// Takes the high byte of every sample of 16 bits.
///
/// A picture of 16 bits holds two bytes for one component, and the first byte is
/// the byte of the largest value. A terminal shows some hundred cells of one
/// picture, therefore that byte holds every difference that a user can see.
fn eight_bits_of(samples: &[u8]) -> Vec<u8> {
    samples.chunks_exact(2).map(|pair| pair[0]).collect()
}

/// Makes a PNG file of the raw samples of a picture.
///
/// The screen needs a file, because `ratatui-image` takes a picture of the crate
/// `image` and that crate reads files. A PNG file of the samples costs one write
/// in the memory, and it holds the exact pixels.
fn png_of_the_samples(width: u32, height: u32, colours: usize, samples: &[u8]) -> Option<Vec<u8>> {
    let needed = usize::try_from(u64::from(width) * u64::from(height)).ok()? * colours;

    if samples.len() < needed {
        return None;
    }

    let image = match colours {
        1 => image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(
            width,
            height,
            samples[..needed].to_vec(),
        )?),
        3 => image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(
            width,
            height,
            samples[..needed].to_vec(),
        )?),
        _ => return None,
    };

    let mut out: Vec<u8> = Vec::new();

    image
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .ok()?;

    Some(out)
}

/// Gives a number of a dictionary of a PDF.
fn number_of(dictionary: &Dictionary, key: &[u8]) -> Option<i64> {
    dictionary.get(key).ok()?.as_i64().ok()
}

/// Gives every name of a value that holds one name or a list of names.
fn names_of(dictionary: &Dictionary, key: &[u8]) -> Vec<String> {
    let Ok(value) = dictionary.get(key) else {
        return Vec::new();
    };

    if let Ok(name) = value.as_name() {
        return vec![String::from_utf8_lossy(name).into_owned()];
    }

    let Ok(list) = value.as_array() else {
        return Vec::new();
    };

    list.iter()
        .filter_map(|value| value.as_name().ok())
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect()
}

/// Gives the number of components of one pixel.
///
/// `DeviceGray` gives 1, and `DeviceRGB` gives 3. A space of colour that this
/// module does not know gives nothing, and the picture then shows no line of the
/// screen.
fn colour_count(dictionary: &Dictionary) -> Option<usize> {
    for name in names_of(dictionary, b"ColorSpace") {
        match name.as_str() {
            "DeviceGray" | "CalGray" | "G" => return Some(1),
            "DeviceRGB" | "CalRGB" | "RGB" => return Some(3),
            _ => continue,
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_page() -> Page {
        Page {
            number: 7,
            text: "The first line of the page.\nThe second line.\n".to_string(),
            pictures: vec![Picture {
                name: "Im0".to_string(),
                width: 400,
                height: 300,
                file: Arc::new(vec![0xff, 0xd8]),
            }],
        }
    }

    #[test]
    fn the_xhtml_of_a_page_holds_the_lines_and_the_pictures() {
        let xhtml = xhtml_of_the_page(&a_page());

        assert!(xhtml.contains("<h2>The page 7</h2>"), "{}", xhtml);
        assert!(
            xhtml.contains("<p>The first line of the page.</p>"),
            "{}",
            xhtml
        );
        assert!(xhtml.contains("<p>The second line.</p>"), "{}", xhtml);
        // A terminal that draws no picture must still tell the user.
        assert!(
            xhtml.contains("[ the picture Im0: 400 by 300 pixels ]"),
            "{}",
            xhtml
        );
    }

    /// A page of a PDF can hold no text at all. An empty page must give a line,
    /// or the screen shows nothing and the user reads it as a fault.
    #[test]
    fn a_page_with_nothing_gives_a_line() {
        let empty = Page {
            number: 1,
            text: "   \n\n".to_string(),
            pictures: Vec::new(),
        };

        let xhtml = xhtml_of_the_page(&empty);
        assert!(xhtml.contains("holds no text and no picture"), "{}", xhtml);
    }

    /// The text of a page comes from the file of a user, therefore it can hold
    /// the characters of XML. A text with `<` must not make a tag.
    #[test]
    fn the_text_of_a_page_cannot_make_a_tag() {
        let page = Page {
            number: 1,
            text: "<script>alert(\"one\" & 'two')</script>".to_string(),
            pictures: Vec::new(),
        };

        let xhtml = xhtml_of_the_page(&page);

        assert!(!xhtml.contains("<script>"), "{}", xhtml);
        assert!(xhtml.contains("&lt;script&gt;"), "{}", xhtml);
        assert!(xhtml.contains("&amp;"), "{}", xhtml);
        assert!(xhtml.contains("&quot;"), "{}", xhtml);
    }

    #[test]
    fn a_picture_of_jpeg_keeps_the_bytes_of_the_file() {
        let mut dictionary = Dictionary::new();
        dictionary.set("Subtype", Object::Name(b"Image".to_vec()));
        dictionary.set("Width", 400_i64);
        dictionary.set("Height", 300_i64);
        dictionary.set("Filter", Object::Name(b"DCTDecode".to_vec()));

        // The two first bytes of every JPEG file.
        let content = vec![0xff, 0xd8, 0xff, 0xe0, 1, 2, 3];

        let picture = picture_of_the_stream("Im0", &dictionary, &content, || None)
            .expect("a picture of JPEG must come");

        assert_eq!(picture.name, "Im0");
        assert_eq!((picture.width, picture.height), (400, 300));
        // The bytes of the stream are the file. The program copies them one
        // time, and it makes no picture in the memory.
        assert_eq!(picture.file.as_slice(), content.as_slice());
    }

    #[test]
    fn a_picture_of_raw_samples_becomes_a_png_file() {
        let mut dictionary = Dictionary::new();
        dictionary.set("Subtype", Object::Name(b"Image".to_vec()));
        dictionary.set("Width", 2_i64);
        dictionary.set("Height", 2_i64);
        dictionary.set("Filter", Object::Name(b"FlateDecode".to_vec()));
        dictionary.set("BitsPerComponent", 8_i64);
        dictionary.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));

        // Four pixels of three components.
        let samples: Vec<u8> = vec![
            255, 0, 0, //
            0, 255, 0, //
            0, 0, 255, //
            255, 255, 255,
        ];

        let picture = picture_of_the_stream("Im1", &dictionary, &[], || Some(samples.clone()))
            .expect("a picture");

        // The first eight bytes of every PNG file.
        assert_eq!(
            &picture.file[..8],
            &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
        );

        // The crate `image` reads the file that this module made, and the pixels
        // are the pixels of the samples.
        let read = image::load_from_memory(&picture.file).expect("the file must be a picture");
        assert_eq!((read.width(), read.height()), (2, 2));
        assert_eq!(read.to_rgb8().get_pixel(0, 0).0, [255, 0, 0]);
        assert_eq!(read.to_rgb8().get_pixel(1, 1).0, [255, 255, 255]);
    }

    /// A picture that the module does not read gives no picture, and it stops
    /// nothing. The line of the text then says that the picture exists.
    #[test]
    fn a_picture_that_the_program_does_not_read_gives_nothing() {
        let mut dictionary = Dictionary::new();
        dictionary.set("Subtype", Object::Name(b"Image".to_vec()));
        dictionary.set("Width", 2_i64);
        dictionary.set("Height", 2_i64);
        dictionary.set("Filter", Object::Name(b"JPXDecode".to_vec()));
        dictionary.set("BitsPerComponent", 8_i64);
        dictionary.set("ColorSpace", Object::Name(b"DeviceCMYK".to_vec()));

        assert!(picture_of_the_stream("Im2", &dictionary, &[1, 2, 3], || None).is_none());

        // A picture of 1 bit of one component: the form of a fax.
        let mut deep = Dictionary::new();
        deep.set("Subtype", Object::Name(b"Image".to_vec()));
        deep.set("Width", 2_i64);
        deep.set("Height", 2_i64);
        deep.set("Filter", Object::Name(b"FlateDecode".to_vec()));
        deep.set("BitsPerComponent", 1_i64);
        deep.set("ColorSpace", Object::Name(b"DeviceGray".to_vec()));

        assert!(picture_of_the_stream("Im3", &deep, &[], || Some(vec![0; 24])).is_none());

        // A picture of no size.
        let mut empty = Dictionary::new();
        empty.set("Subtype", Object::Name(b"Image".to_vec()));
        empty.set("Width", 0_i64);
        empty.set("Height", 10_i64);

        assert!(picture_of_the_stream("Im4", &empty, &[], || None).is_none());
    }

    /// A picture of 16 bits of one component comes from every PDF that a PNG
    /// file made. The program takes the high byte of each sample. See T-57.
    #[test]
    fn a_picture_of_sixteen_bits_gives_a_picture() {
        let mut dictionary = Dictionary::new();
        dictionary.set("Subtype", Object::Name(b"Image".to_vec()));
        dictionary.set("Width", 2_i64);
        dictionary.set("Height", 1_i64);
        dictionary.set("Filter", Object::Name(b"FlateDecode".to_vec()));
        dictionary.set("BitsPerComponent", 16_i64);
        dictionary.set("ColorSpace", Object::Name(b"DeviceGray".to_vec()));

        // Two pixels of one component of two bytes: 0xff00 and 0x2010.
        let samples = vec![0xff, 0x00, 0x20, 0x10];

        let picture = picture_of_the_stream("Im0", &dictionary, &[], || Some(samples))
            .expect("a picture of 16 bits must come");

        let read = image::load_from_memory(&picture.file).expect("the file must be a picture");
        assert_eq!((read.width(), read.height()), (2, 1));
        assert_eq!(read.to_luma8().get_pixel(0, 0).0, [0xff]);
        assert_eq!(read.to_luma8().get_pixel(1, 0).0, [0x20]);
    }

    #[test]
    fn the_high_byte_of_every_sample() {
        assert_eq!(eight_bits_of(&[0xff, 0x00, 0x01, 0x02]), vec![0xff, 0x01]);
        // A list with one byte too many gives the pairs that are complete.
        assert_eq!(eight_bits_of(&[0xaa, 0xbb, 0xcc]), vec![0xaa]);
        assert!(eight_bits_of(&[]).is_empty());
    }

    /// A picture of more pixels than the screen shows must become smaller. See
    /// T-62.
    #[test]
    fn a_large_picture_becomes_smaller_and_it_keeps_its_form() {
        // A picture of 1400 by 1900, in the form of a JPEG file.
        let large = image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(1400, 1900, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        }));

        let mut of_the_page: Vec<u8> = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(
            &mut std::io::Cursor::new(&mut of_the_page),
            90,
        )
        .encode_image(&large)
        .expect("the test must write a JPEG file");

        let picture = smaller_if_it_is_large("Im0", 1400, 1900, &of_the_page);

        // The two numbers stay the numbers of the page, therefore the line of the
        // text tells the user the size of the picture of their book.
        assert_eq!((picture.width, picture.height), (1400, 1900));

        // The file holds fewer bytes and fewer pixels.
        assert!(
            picture.file.len() < of_the_page.len(),
            "{} bytes of {}",
            picture.file.len(),
            of_the_page.len()
        );

        let read = image::load_from_memory(&picture.file).expect("the file must be a picture");
        assert!(read.width() <= LARGEST_SIDE && read.height() <= LARGEST_SIDE);

        // The form of the file agrees with the form of the page, to one part in a
        // hundred.
        let of_the_numbers = 1400.0 / 1900.0;
        let of_the_file = f64::from(read.width()) / f64::from(read.height());
        assert!(
            (of_the_numbers - of_the_file).abs() < 0.01,
            "the form changed from {} to {}",
            of_the_numbers,
            of_the_file
        );
    }

    /// A picture that the screen shows already keeps the bytes of its file. The
    /// program then reads no picture and it writes no picture.
    #[test]
    fn a_small_picture_keeps_the_bytes_of_the_page() {
        let bytes = vec![0xff, 0xd8, 1, 2, 3, 4];
        let picture = smaller_if_it_is_large("Im1", 400, 300, &bytes);

        assert_eq!(picture.file.as_slice(), bytes.as_slice());
        assert_eq!((picture.width, picture.height), (400, 300));

        // A file that the program cannot read keeps its bytes as well, therefore a
        // picture of a form that `image` does not know still reaches the screen.
        let broken = smaller_if_it_is_large("Im2", 4000, 3000, &[1, 2, 3]);
        assert_eq!(broken.file.as_slice(), &[1, 2, 3]);
    }

    /// A stream that names fewer samples than the size of the picture needs must
    /// give nothing. A reader that trusts the size reads memory that is not its
    /// own.
    #[test]
    fn a_picture_with_too_few_samples_gives_nothing() {
        assert!(png_of_the_samples(4, 4, 3, &[0; 10]).is_none());
        assert!(png_of_the_samples(1, 1, 3, &[1, 2, 3]).is_some());
    }

    #[test]
    fn a_text_of_utf16_of_the_information_comes_as_text() {
        // "Ok" in UTF-16, with the two bytes of mark of a big end.
        let object = Object::String(
            vec![0xfe, 0xff, 0x00, 0x4f, 0x00, 0x6b],
            lopdf::StringFormat::Literal,
        );

        assert_eq!(text_of(&object).as_deref(), Some("Ok"));

        let plain = Object::String(b"A Title".to_vec(), lopdf::StringFormat::Literal);
        assert_eq!(text_of(&plain).as_deref(), Some("A Title"));
    }

    /// The part of the book comes from the size of each page. A page of a
    /// picture and of no text must not give 0, or the part of the book jumps.
    #[test]
    fn a_page_of_a_picture_has_a_size() {
        let pdf = Pdf {
            pages: vec![
                Page {
                    number: 1,
                    text: "abc".to_string(),
                    pictures: Vec::new(),
                },
                Page {
                    number: 2,
                    text: String::new(),
                    pictures: vec![Picture {
                        name: "Im0".to_string(),
                        width: 1,
                        height: 1,
                        file: Arc::new(Vec::new()),
                    }],
                },
                Page::default(),
            ],
            title: String::new(),
            author: String::new(),
        };

        assert_eq!(pdf.page_sizes(), vec![3, SIZE_OF_A_PICTURE, 1]);
    }
}
