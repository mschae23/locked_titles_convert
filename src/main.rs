// This file is part of locked_titles_convert.
//
// locked_titles_convert is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;
use anyhow::{anyhow, bail, Context, Result};

pub fn main() -> Result<(), anyhow::Error> {
    let exported: serde_json::Value = serde_json::from_reader(std::io::stdin())?;
    let messages = exported.get("messages").with_context(|| "#dearrow-locked-titles export does not contain messages")?
        .as_array().with_context(|| "#dearrow-locked-titles export \"messages\" field is not an array")?;

    std::fs::create_dir_all("./output")?;
    let mut users = csv::Writer::from_path("./output/users.csv")?;
    let mut votes = csv::Writer::from_path("./output/votes.csv")?;

    users.write_record(&["internal_user_id", "user_id", "username"])?;
    votes.write_record(&["internal_vote_id", "submitter", "timestamp", "video_id", "original_title", "locked_title", "new_title", "locked_votes", "new_votes"])?;

    users.write_record(&["0", "6653aafce754bdc5c5d5d2ad4f7ef05e9d7eb91e29f9b8ab9b7a427e63a95254", "mschae23"])?;
    users.write_record(&["1", "7b89ea26f77bda8176e655eee86029f28c1e6514b6d6e3450bce362b5b126ca3", "Ajay"])?;
    users.write_record(&["2", "20bb74bc59a43defbd8995e0e7c675fbabd375dfe79ed0d0456e363ba6089633", "blabdude"])?;
    users.write_record(&["3", "6bf14387297df3152584ae95d6682cdc98cfee30c1e8a6a4ad876babdfee555b", "jiraph"])?;

    let mut user_map: HashMap<String, i32> = HashMap::with_capacity(4);
    user_map.insert(String::from("6653aafce754bdc5c5d5d2ad4f7ef05e9d7eb91e29f9b8ab9b7a427e63a95254"), 0);
    user_map.insert(String::from("7b89ea26f77bda8176e655eee86029f28c1e6514b6d6e3450bce362b5b126ca3"), 1);
    user_map.insert(String::from("20bb74bc59a43defbd8995e0e7c675fbabd375dfe79ed0d0456e363ba6089633"), 2);
    user_map.insert(String::from("6bf14387297df3152584ae95d6682cdc98cfee30c1e8a6a4ad876babdfee555b"), 3);

    let mut internal_user_id = 4;
    let mut internal_vote_id = 0;

    for message in messages {
        if message["author"]["id"].as_str().ok_or(anyhow!("Author ID is not a string"))? != "1167323752363733074" {
            continue;
        }

        // let discord_message_link = format!("https://discord.com/channels/603643120093233162/1167321798178766968/{}", message["id"]);

        let embed = message["embeds"].as_array().ok_or(anyhow!("Embeds field is not an array"))?.get(0).ok_or(anyhow!("Embeds array is empty"))?;

        let video_link = embed["url"].as_str().ok_or(anyhow!("URL is not a string"))?;
        let original_title = embed["title"].as_str().ok_or(anyhow!("Title is not a string"))?;
        let text = embed["description"].as_str().ok_or(anyhow!("Description is not a string"))?;

        let mut remaining = text;
        anyhow::ensure!(&remaining[..2] == "**");
        remaining = &remaining[2..];

        let mut i = 0;

        for c in remaining.chars() {
            if c.is_digit(10) {
                i += c.len_utf8();
            } else if c == '*' {
                break;
            } else {
                bail!("Invalid embed description format");
            }
        }

        let locked_votes = remaining[..i].parse::<i32>()?;
        remaining = &remaining[i..];
        anyhow::ensure!(&remaining[..14] == "** Votes vs **");
        remaining = &remaining[14..];

        i = 0;

        for c in remaining.chars() {
            if c.is_digit(10) {
                i += c.len_utf8();
            } else if c == '*' {
                break;
            } else {
                bail!("Invalid embed description format");
            }
        }

        let new_votes = remaining[..i].parse::<i32>()?;
        remaining = &remaining[i..];
        anyhow::ensure!(&remaining[..2] == "**");
        remaining = &remaining[2..];

        while remaining.chars().next().ok_or(anyhow!("Embed description is not long enough"))?.is_whitespace() {
            remaining = &remaining[1..];
        }

        anyhow::ensure!(&remaining[..18] == "**Locked title:** ");
        remaining = &remaining[18..];

        i = 0;

        for c in remaining.chars() {
            if c == '\n' {
                break;
            } else {
                i += c.len_utf8();
            }
        }

        anyhow::ensure!(i != 0);
        let locked_title = remaining[..i].trim();
        remaining = &remaining[i..];
        anyhow::ensure!(&remaining[..16] == "\n**New title:** ");
        remaining = &remaining[16..];

        i = 0;

        for c in remaining.chars() {
            if c == '\n' {
                break;
            } else {
                i += c.len_utf8();
            }
        }

        anyhow::ensure!(i != 0);
        let new_title = remaining[..i].trim();
        remaining = &remaining[i..];
        anyhow::ensure!(&remaining[..20] == "\n\n**Submitted by:** ");
        remaining = &remaining[20..];

        i = 0;

        for c in remaining.chars() {
            if c == '\n' {
                break;
            } else {
                i += c.len_utf8();
            }
        }

        let submitter_username = &remaining[..i];
        remaining = &remaining[i..];
        anyhow::ensure!(&remaining[..1] == "\n");
        remaining = &remaining[1..];

        let submitter_userid = remaining;

        let submitter_internal_user_id = if let Some(user_id) = user_map.get(submitter_userid) {
            *user_id
        } else {
            let user_id = internal_user_id;
            users.write_record(&[&user_id.to_string(), submitter_userid, submitter_username])?;
            user_map.insert(submitter_userid.to_string(), user_id);
            internal_user_id += 1;
            user_id
        };

        let timestamp = chrono::DateTime::parse_from_str(message["timestamp"].as_str()
            .ok_or(anyhow!("Timestamp field is not a string"))?, "%Y-%m-%dT%H:%M:%S%.f%:z")?.timestamp();

        votes.write_record(&[&internal_vote_id.to_string(), &submitter_internal_user_id.to_string(), &timestamp.to_string(), &video_link[32..],
            original_title, locked_title, new_title, &locked_votes.to_string(), &new_votes.to_string()])?;
        internal_vote_id += 1;
    }

    votes.flush()?;
    users.flush()?;
    Ok(())
}
