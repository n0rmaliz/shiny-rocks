use urlencoding::encode;
use rand::thread_rng;
use rand::seq::{IteratorRandom, SliceRandom};
use reqwest::{Result as ReqResult};
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

static COUNTER: AtomicUsize = AtomicUsize::new(1);
fn get_id() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }

const SEARCH_QUERIES: [&str; 5] = [
    "shiny thing",
    "shiny rock",
    "rock",
    "gemstone",
    "meteorite",
];

pub struct ShinyThing {
    id: usize,
    pub name: String,
    pub shinyness: f32,
    pub url: String,
}

impl ShinyThing {
    pub fn new(id: usize, name: String, url: String) -> Self {
        Self { id, name, shinyness: 0., url }
    }

    pub async fn gen_new() -> ReqResult<Self> {
        // why is cors yelling at me
        let mut rng = thread_rng();

        let query = SEARCH_QUERIES
            .choose(&mut rng)
            .expect("unable to generate search query");
        let url = format!(
            "https://www.google.com/search?q={}&tbm=isch",
            encode(query),
        );
        println!("{}", url);

        let html = Html::parse_document(
            reqwest::get(url)
                .await?
                .text()
                .await?
                .as_str()
        );
        let selector = Selector::parse("img").expect("unable to parse html selector");
        let [chosen_img_url, chosen_img_name] = html.select(&selector).map(|elem| {
            let elem_attrs: HashMap<&str, &str> = elem.value()
                .attrs()
                .collect();
            let to_find = ["src", "alt"];
            to_find.map(|k| {
                elem_attrs
                    .get(k)
                    .map_or_else(
                        || panic!("unable to find tag {k} in {elem:?}"),
                        |&a| a.to_string(),
                    )
            })
        }).choose(&mut rng).expect("unable to get image");
        println!("{}, {}", chosen_img_url, chosen_img_name);

        let mut file = File::create(format!(
            "assets/imgs/{}.png",
            chosen_img_name,
        )).expect("unable to create asset file");
        let img_bytes = reqwest::get(&chosen_img_url)
            .await?
            .bytes()
            .await?;
        file.write_all(
            img_bytes
                .iter()
                .as_slice()
        ).expect("unable to write image");

        ReqResult::Ok(Self {
            id: get_id(),
            name: chosen_img_name,
            shinyness: 0.,
            url: chosen_img_url,
        })
    }
}