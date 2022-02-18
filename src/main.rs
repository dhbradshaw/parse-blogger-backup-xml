use parse_blogger_backup_xml::get_posts;

/// Parse the backup.xml file from a Google Blogger backup.
///
fn main() {
    // Read backup file location from command line
    let args: Vec<String> = std::env::args().collect();
    let backup_file_path = &args[1];
    let posts = get_posts(backup_file_path).unwrap();

    // Print out posts
    println!("{:#?}", &posts);

    // Print out the posts length
    let post_count = posts.len();
    println!("\n{post_count} posts in total");

    if !posts.is_empty() {
        let first_post_published = &posts[0].published;
        let last_post_published = &posts[post_count - 1].published;
        println!("published from {first_post_published} to {last_post_published}");
    }
}
