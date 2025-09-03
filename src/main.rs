use anyhow::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut csv = csv::Writer::from_path("baidu_pan_records.csv")?;
    let body = reqwest::get("https://www.qingju.org").await?.text().await?;
    let dom = tl::parse(&body, Default::default())?;
    let parser = dom.parser();

    csv.write_record(["Title", "Mirror 1", "Mirror 2", "Mirror 3", "Mirror 4"])?;

    if let Some(last) = dom
        .query_selector("a.page-numbers")
        .context(line!())?
        .filter_map(|handle| handle.get(parser))
        .filter_map(|node| node.inner_text(parser).parse().ok())
        .last()
    {
        let mut posts = Vec::with_capacity(last);

        posts.push(find_posts(&body)?);

        for i in 2..=last {
            let url = format!("https://www.qingju.org/page/{i}");
            let body = reqwest::get(url).await?.text().await?;

            posts.push(find_posts(&body)?);
        }

        for (title, url) in posts.into_iter().flat_map(|posts| posts.into_iter()) {
            let mut record = vec![Default::default(); 5];
            let body = reqwest::get(url).await?.text().await?;
            let dom = tl::parse(&body, Default::default())?;
            let parser = dom.parser();

            record[0] = title;

            for (mirror, i) in dom
                .query_selector("a[href]")
                .context(line!())?
                .filter_map(|handle| handle.get(parser))
                .filter_map(|node| node.as_tag())
                .filter_map(|tag| tag.attributes().get("href").flatten())
                .map(|url| url.as_utf8_str().to_string())
                .filter(|url| url.starts_with("https://pan.baidu.com"))
                .zip(1..5)
            {
                record[i] = mirror;
            }

            csv.write_record(record)?;
        }
    }

    Ok(csv.flush()?)
}

fn find_posts(body: &str) -> Result<Vec<(String, String)>> {
    let dom = tl::parse(body, Default::default())?;
    let parser = dom.parser();

    Ok(dom
        .query_selector("posts.posts-item")
        .context(line!())?
        .filter_map(|handle| handle.get(parser))
        .filter_map(|node| node.as_tag())
        .filter_map(|tag| tag.query_selector(parser, "h2.item-heading"))
        .filter_map(|mut query| query.next())
        .filter_map(|handle| handle.get(parser))
        .filter_map(|node| node.children())
        .filter_map(|children| children.all(parser).first())
        .filter_map(|node| node.as_tag())
        .filter_map(|tag| {
            tag.attributes().get("href").flatten().map(|url| {
                (
                    tag.inner_text(parser).to_string(),
                    url.as_utf8_str().to_string(),
                )
            })
        })
        .collect())
}
