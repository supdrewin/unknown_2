use anyhow::*;
use base64::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut csv = csv::Writer::from_path("baidu_pan_records.csv")?;
    let mut posts = Vec::with_capacity(92);
    let client = reqwest::Client::builder().cookie_store(true).build()?;

    csv.write_record(["Title", "URL"])?;

    for i in 1..=92 {
        let url = format!("https://game.acgs.one/page/{i}");
        let body = client.get(url).send().await?.text().await?;

        posts.push(find_posts(&body)?);
    }

    for (title, url) in posts.into_iter().flat_map(|posts| posts.into_iter()) {
        let mut body = client.get(&url).send().await?.text().await?;
        let mut refresh = false;

        {
            let dom = tl::parse(&body, Default::default())?;
            let parser = dom.parser();

            if let Some(_) = dom
                .query_selector("div[role]")
                .context(line!())?
                .filter_map(|handle| handle.get(parser))
                .filter_map(|node| node.as_tag())
                .filter_map(|tag| tag.attributes().get("role").flatten())
                .filter(|data| data.as_bytes().eq(b"alert"))
                .next()
            {
                client
                    .post(format!("{url}?themeAction=comment"))
                    .multipart(
                        reqwest::multipart::Form::new()
                            .text("text", "Thanks!")
                            .text("author", "guest")
                            .text("mail", "guest@gmail.com")
                            .text("sum", "43")
                            .text("num1", "21")
                            .text("num2", "22"),
                    )
                    .send()
                    .await?;

                refresh = true;
            }
        }

        if refresh {
            body = client.get(url).send().await?.text().await?;
        }

        let dom = tl::parse(&body, Default::default())?;
        let parser = dom.parser();

        if let Some(url) = dom
            .query_selector("div[x-data]")
            .context(line!())?
            .filter_map(|handle| handle.get(parser))
            .filter_map(|node| node.as_tag())
            .filter_map(|tag| tag.attributes().get("x-data").flatten())
            .filter_map(|data| data.try_as_utf8_str())
            .filter_map(|data| data.strip_prefix("{url:'"))
            .map(|data| data.split_at(data.len() - 3).0)
            .filter_map(|input| BASE64_STANDARD.decode(input).ok())
            .filter_map(|vec| String::from_utf8(vec).ok())
            .next()
        {
            let body = reqwest::get(url).await?.text().await?;
            let dom = tl::parse(&body, Default::default())?;
            let parser = dom.parser();

            if let Some(url) = dom
                .query_selector("a")
                .context(line!())?
                .filter_map(|handle| handle.get(parser))
                .filter_map(|node| node.as_tag())
                .filter_map(|tag| tag.attributes().get("href").flatten())
                .map(|url| url.as_utf8_str().to_string())
                .next()
            {
                csv.write_record([title, url])?;
            }
        }
    }

    Ok(csv.flush()?)
}

fn find_posts(body: &str) -> Result<Vec<(String, String)>> {
    let dom = tl::parse(body, Default::default())?;
    let parser = dom.parser();

    Ok(dom
        .query_selector("article")
        .context(line!())?
        .filter_map(|handle| handle.get(parser))
        .filter_map(|node| node.as_tag())
        .filter_map(|tag| tag.query_selector(parser, "a"))
        .filter_map(|mut query| query.next())
        .filter_map(|handle| handle.get(parser))
        .filter_map(|node| node.as_tag())
        .map(|tag| tag.attributes())
        .filter_map(|attributes| {
            attributes
                .get("title")
                .flatten()
                .filter(|title| !title.as_bytes().ends_with(b"</span>"))
                .zip(attributes.get("href").flatten())
                .into_iter()
                .map(|(title, url)| {
                    (
                        title.as_utf8_str().to_string(),
                        url.as_utf8_str().to_string(),
                    )
                })
                .next()
        })
        .collect())
}
