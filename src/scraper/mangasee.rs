use crate::scraper::{Chapter, ChapterNumber, Manga, Params, Scraping};
use regex::Regex;
use std::borrow::BorrowMut;
use std::collections::BTreeMap;

#[derive(Copy, Clone)]
pub struct Mangasee {
    pub url: &'static str,
}

impl Default for Mangasee {
    fn default() -> Self {
        return Mangasee {
            url: "https://mangaseeonline.us",
        };
    }
}

impl Scraping for Mangasee {
    fn new(url: &'static str) -> Mangasee {
        return Mangasee { url: &url };
    }

    fn get_mangas(&self, param: Params) -> Vec<Manga> {
        let mut mangas: Vec<Manga> = Vec::new();

        let params = vec![
            ("keyword".to_owned(), param.keyword.to_owned()),
            ("page".to_owned(), param.page.to_owned()),
            ("sortBy".to_owned(), param.sort_by.to_owned()),
            ("sortOrder".to_owned(), param.sort_order.to_owned()),
        ];

        let urlencoded = serde_urlencoded::to_string(params).unwrap();

        let resp = ureq::post(format!("{}/search/request.php", self.url).as_str())
            .set(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=utf-8",
            )
            .send_string(&urlencoded);

        let html = resp.into_string().unwrap();
        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse(".requested .row").unwrap();
        for row in document.select(&selector) {
            let mut manga = Manga {
                title: String::from(""),
                author: String::from(""),
                genre: vec![],
                status: String::from(""),
                description: String::from(""),
                url: String::from(""),
                thumbnail_url: String::from(""),
                chapters: vec![],
            };

            let sel = scraper::Selector::parse("img").unwrap();
            for el in row.select(&sel) {
                manga.thumbnail_url = el.value().attr("src").unwrap().to_owned();
            }

            let sel = scraper::Selector::parse(".resultLink").unwrap();
            for el in row.select(&sel) {
                manga.title = el.inner_html();
                manga.url = el.value().attr("href").unwrap().to_owned();
            }
            mangas.push(manga);
        }

        return mangas;
    }

    fn get_latest_mangas(&self) -> Vec<Manga> {
        let resp = ureq::get("https://mangaseeonline.us").call();
        let html = resp.into_string().expect("failed to get page");

        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse(".latestSeries").unwrap();

        let mut latest_mangas: Vec<Manga> = Vec::new();
        for element in document.select(&selector) {
            let link = element.value().attr("href").unwrap();
            let re = Regex::new(r"-chapter-.*").unwrap();
            let link = re.replace_all(link, "");

            let manga = Manga {
                title: String::from(""),
                author: String::from(""),
                genre: vec![],
                status: String::from(""),
                description: String::from(""),
                url: String::from(link).replace("read-online", "manga"),
                thumbnail_url: String::from(""),
                chapters: Default::default(),
            };
            latest_mangas.push(manga)
        }

        return latest_mangas;
    }

    fn get_manga_info(&self, path: String) -> Manga {
        let mut m = Manga {
            title: "".to_string(),
            author: "".to_string(),
            genre: vec![],
            status: "".to_string(),
            description: "".to_string(),
            url: path.to_owned(),
            thumbnail_url: "".to_string(),
            chapters: Default::default(),
        };

        let resp = ureq::get(format!("{}{}", self.url, path.to_owned()).as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse(".leftImage img").unwrap();
        for element in document.select(&selector) {
            let src = element.value().attr("src").unwrap();
            m.thumbnail_url = String::from(src);
        }

        let selector = scraper::Selector::parse("h1[class=\"SeriesName\"]").unwrap();
        for element in document.select(&selector) {
            m.title = element.inner_html();
        }

        let selector = scraper::Selector::parse("a[href*=\"author\"]").unwrap();

        for element in document.select(&selector) {
            for text in element.text() {
                m.author = String::from(text);
            }
        }

        let selector = scraper::Selector::parse("a[href*=\"genre\"]").unwrap();
        for element in document.select(&selector) {
            for text in element.text() {
                m.genre.push(String::from(text));
            }
        }

        let selector = scraper::Selector::parse(".PublishStatus").unwrap();
        for element in document.select(&selector) {
            let status = element.value().attr("status").unwrap();
            m.status = String::from(status);
        }

        let selector = scraper::Selector::parse(".description").unwrap();
        for element in document.select(&selector) {
            for text in element.text() {
                m.description = String::from(text);
            }
        }

        let mut next_chapter = "".to_string();
        let selector = scraper::Selector::parse(".mainWell .chapter-list a[chapter]").unwrap();
        for element in document.select(&selector) {
            let rank = String::from(element.value().attr("chapter").unwrap());
            let link = element.value().attr("href").unwrap();

            m.chapters.push(rank);
        }
        return m;
    }

    fn get_chapter(&self, path: String) -> Chapter {
        let resp = ureq::get(format!("{}{}", self.url, path.to_owned()).as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);

        let mut chapter = Chapter::default();

        let selector = scraper::Selector::parse(".fullchapimage img").unwrap();
        for element in document.select(&selector) {
            chapter
                .pages
                .push(String::from(element.value().attr("src").unwrap()));
        }
        return chapter;
    }
}
