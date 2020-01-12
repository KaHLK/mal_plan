use super::{Item, ItemMediaType, ListType, Sort};

use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manga {
    id: u32,
    status: u8,
    manga_id: u32,
    manga_num_chapters: u16,
    manga_publishing_status: u8,
    manga_url: String,
    manga_media_type_string: MangaType,
}

#[derive(Debug, Serialize, Deserialize)]
enum MangaType {
    Manga,
    Novel,
    #[serde(rename = "One-shot")]
    OneShot,
    Doujinshi,
    Manhwa,
    Manhua,
}

impl From<&Manga> for Item {
    fn from(manga: &Manga) -> Item {
        Item {
            item_type: ListType::Manga,
            id: manga.manga_id,
            amount: manga.manga_num_chapters,
            publishing_status: manga.manga_publishing_status,
            url: manga.manga_url.clone(),
            media_type: match manga.manga_media_type_string {
                MangaType::Manga => ItemMediaType::Manga,
                MangaType::Novel => ItemMediaType::Novel,
                MangaType::OneShot => ItemMediaType::OneShot,
                MangaType::Doujinshi => ItemMediaType::Doujinshi,
                MangaType::Manhwa => ItemMediaType::Manhwa,
                MangaType::Manhua => ItemMediaType::Manhua,
            },
        }
    }
}

fn sort_to_manga_column(sort: &Sort) -> i8 {
    match sort {
        Sort::Asc => -9,
        Sort::Desc => 9,
    }
}

pub fn fetch_all<F>(user: String, sort: Sort, fun: F) -> Result<Vec<Manga>, Box<dyn Error>>
where
    F: Fn(usize) -> (),
{
    let mut offset: usize = 0;
    let mut list: Vec<Manga> = Vec::with_capacity(300);
    loop {
        fun(offset);
        let mut manga = fetch_data(&user, &sort, offset as u16)?;
        if manga.len() == 0 {
            break;
        }
        list.append(&mut manga);
        offset = list.len();
    }

    Ok(list)
}

pub fn fetch_data(user: &String, sort: &Sort, offset: u16) -> Result<Vec<Manga>, Box<dyn Error>> {
    let url = format!(
        "https://myanimelist.net/mangalist/{user}/load.json?status=6&order={order}&offset={offset}",
        user = user,
        order = sort_to_manga_column(&sort),
        offset = offset
    );

    let res = attohttpc::get(url).send()?.text()?;
    let manga: Vec<Manga> = serde_json::from_str(&res)?;

    Ok(manga)
}
