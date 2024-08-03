use lettre::{
    message::{header::{self, ContentType}, Attachment, MultiPart, SinglePart},
    SmtpTransport, Message, Transport,
};
use lettre::transport::smtp::authentication::Credentials;
use std::fs;
use std::io::{self, Write};
use structopt::StructOpt;
use chrono::Local;
use std::io::Read;
use std::fs::OpenOptions;
use std::path::Path;

#[derive(Debug, StructOpt)]
#[structopt(name = "courier | 信使", about = "sent email.", author = "Enomothem", version = "0.2")]
struct Opt {
    /// 指定账号
    #[structopt(short, long, value_name = "xxx@xxx.com")]
    user: String,
    
    /// 指定密码
    #[structopt(short, long, value_name = "key")]
    key: String,

    /// 指定收件人
    #[structopt(short, long, value_name = "xxx@xxx.com")]
    to: String,

    /// 指定报告路径
    #[structopt(short, long, value_name = "output.html")]
    report: String,

    /// 指定服务器
    #[structopt(short, long, value_name = "xxx.com")]
    server: String,

    /// 指定抄送人（可选）
    #[structopt(short, long, value_name = "xxx@xxx.com" )]
    cc: Option<String>,

    /// 指定密送人（可选）
    #[structopt(short, long, value_name = "xxx@xxx.com")]
    bcc: Option<String>,
}

fn read_email_to_html(path: &str) -> String {
    // Open the file
    let mut email_file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return String::new();
        }
    };

    let mut email_bytes = Vec::new();
    if let Err(e) = email_file.read_to_end(&mut email_bytes) {
        eprintln!("Unable to read file: {}", e);
        return String::new();
    }

    String::from_utf8_lossy(&email_bytes).to_string()
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    // 检查是否存在日志文件
    if !Path::new("output").exists() {
        fs::create_dir_all("output")?;
    }
    // 指定要追加的文件路径
    let file_path = "output/courier.log";

    // 打开文件用于追加，如果文件不存在则创建它
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)  // 设置为追加模式
        .open(file_path)?;


    // 邮件的附件构造
    let attachment = Attachment::new(opt.report.clone().into())
        .body(
            fs::read(&opt.report).expect("读取html失败"),
            ContentType::parse("text/html").expect("解析html格式失败:")
        );

    let html_content = read_email_to_html(&opt.report);
    // 邮件的参数
    let from_address = opt.user.clone();
    let to_address = opt.to.clone();
    let cc_address = opt.cc.clone();
    let bcc_address = opt.bcc;

    // 在subject标题中加入时间戳，转为日期格式
    let now = Local::now();
    let stime = now.format("%Y-%m-%d %H:%M:%S").to_string();


    let subject = format!("Linux应急响应报告 - {}", stime);

    // 认证相关
    let username = String::from(opt.user);
    let password = String::from(opt.key);    // 不是密码，不是密码，是授权码！！！
    let smtp_server = &opt.server;

    // 构造邮件体内容
    let mut email_builder = Message::builder()
        .from(from_address.parse().expect("解析发送邮箱地址失败:"))
        .to(to_address.parse().expect("解析收件邮箱地址失败: "))
        .subject(subject);

    // 处理抄送和密送地址
    if let Some(cc) = cc_address {
        email_builder = email_builder.cc(cc.parse().expect("解析抄送邮箱地址失败: "));
    }
    if let Some(bcc) = bcc_address {
        email_builder = email_builder.bcc(bcc.parse().expect("解析密送邮箱地址失败: "));
    }

    let email = email_builder
        .multipart(
            MultiPart::mixed() // 使用混合模式
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(String::from(html_content)),
                )
                .singlepart(attachment)
        )
        .expect("构造邮件消息体失败:");

    let creds = Credentials::new(username, password);

    // 默认使用TLS连接
    let mailer = SmtpTransport::relay(smtp_server)
        .expect("解析SMTP服务器失败:")
        .credentials(creds)
        .build();
    let time = now.format("%Y-%m-%d %H:%M:%S").to_string();
    // let time = "x";
    // sleep(Duration::from_secs(2));
    match mailer.send(&email) {
        Ok(_) => {
            
            println!("[+] {time} 邮件发送成功!");

            if let Some(cc) = &opt.cc {
                // 如果存在 cc 参数
                writeln!(file, "[+] {} 邮件发送成功！收信人：{}, 抄送人：{}", time, opt.to, cc)?;
            } else {
                // 如果不存在 cc 参数
                writeln!(file, "[+] {} 邮件发送成功！收信人：{}", time, opt.to)?;
            }
        },
        Err(e) => {
            writeln!(file, "[!] {} 邮件发送失败: {:?}", time, e)?;
            eprintln!("发送邮件失败: {:?}", e);
        },
    }

    Ok(())

    
}
