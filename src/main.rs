use anyhow::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut writer = csv::Writer::from_path("baidu_pan_records.csv")?;
    let body = reqwest::get("https://www.qingju.org").await?.text().await?;
    let mut posts = vec![find_posts(&body)?];
    let dom = tl::parse(&body, Default::default())?;
    let parser = dom.parser();

    if let Some(last) = dom
        .query_selector("a.page-numbers")
        .context(line!())?
        .filter_map(|handle| handle.get(parser))
        .filter_map(|node| node.inner_text(parser).parse::<usize>().ok())
        .last()
    {
        for i in 2..=last {
            let url = format!("https://www.qingju.org/page/{i}");
            let body = reqwest::get(url).await?.text().await?;

            posts.push(find_posts(&body)?);
        }

        for (title, url) in posts.into_iter().flat_map(|posts| posts.into_iter()) {
            let mut record = vec![title];
            let body = reqwest::get(url).await?.text().await?;
            let dom = tl::parse(&body, Default::default())?;
            let parser = dom.parser();

            record.extend(
                dom.query_selector("a[href]")
                    .context(line!())?
                    .filter_map(|handle| handle.get(parser))
                    .filter_map(|node| node.as_tag())
                    .filter_map(|tag| tag.attributes().get("href").flatten())
                    .map(|url| url.as_utf8_str().to_string())
                    .filter(|url| url.starts_with("https://pan.baidu.com")),
            );
            writer.write_record(record)?;
        }
    }

    Ok(writer.flush()?)
}

fn find_posts<'a>(body: &str) -> Result<Vec<(String, String)>> {
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
