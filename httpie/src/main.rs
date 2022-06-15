use std::{collections::HashMap, str::FromStr};

use clap::Parser;
use colored::Colorize;
use mime::Mime;
use reqwest::{Client, Url, Response, header};
use anyhow::{anyhow, Result};

#[derive(Debug, Parser)]
#[clap(author = "JadeStrong")] // 指定了一些命令信息，比如 version 是该命令的版本信息，author 是命令的作用等
#[clap(version = "1.0")]
#[clap(about = "Does awesome things.", long_about = None)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
    /// Options name to operate on
    #[clap(value_parser)]
    name: Option<String>,

    /// Turn debugging infomation on
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(clap::Args, Debug)]
#[clap(about = "GET 请求")]
struct Get {
    /// http 请求的 URL
    #[clap(value_parser = parse_url)]
    url: String,
}

#[derive(clap::Args, Debug)]
#[clap(about = "POST 请求")]
struct Post {
    /// http 请求的 URL
    #[clap(value_parser = parse_url)]
    url: String,
    /// http 请求的 body
    #[clap(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let client = Client::new();
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}


fn parse_url(s: &str) -> Result<String> {
    // 解析 s 为 Url 类型，如果出错会打断执行
    let _url: Url = s.parse()?;
    // into() 方法会尝试将该类型转换成推断的类型，这里就死 str -> String
    Ok(s.into())
}


fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

#[derive(Debug, Clone)]
struct KvPair {
    k: String,
    v: String,
}

/// 当我们实现了 FromStr trait 后，可以用 str.parse() 方法将字符串解析成 KvPair
impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            k : (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_header(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

// 打印服务器版本号 + 状态码
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_header(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }

    print!("\n");
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        },
        _ => println!("{}", body),
    }
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}
