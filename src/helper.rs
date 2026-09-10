pub mod Helper{
    use std::{collections::HashMap, path::PathBuf, process::exit};

    static URL:&str = "";
    static HOSTNAME:&str = "";
    static IP:&str = "";
    static PORT:u32 = 5623;
    static DBG_STR:&str = "";


    #[derive(Debug,Clone)]
    pub struct CLI{
        pub dbg:bool,
        pub method: String,
        pub send: HashMap<String,Vec<PathBuf>>,
        pub receive: Vec<String>,
        pub connection:Connection,
        pub srcdir: Option<PathBuf>,
        pub config:Option<String>,
    }

    pub fn Help(){
        println!("PiRelay - File Transfer Client\n");
        println!("USAGE:");
        println!("    pirelay <METHOD> [OPTIONS]\n");
        println!("METHODS:");
        println!("    SEND     Send files to recipients");
        println!("    RECEIVE  Receive files from senders\n");
        println!("OPTIONS:");
        println!("    -d, --debug              Enable debug mode");
        println!("    -s=<path>, --src=<path>  Output directory for received files");
        println!("    -conf=<path>, --config=<path>  Path to config JSON file");
        println!("    -c=(url,ip,port,hostname), --conn=(...)  Connection settings");
        println!("    --send=recipient=(file1,file2,...)  Send files to recipient");
        println!("    --receive=(sender1,sender2,...)     Receive from specific senders\n");
        println!("EXAMPLES:");
        println!("    pirelay SEND --send=alice=(file.txt,doc.pdf)");
        println!("    pirelay RECEIVE --receive=(alice,bob)");
        println!("    pirelay RECEIVE -s=./downloads");
        exit(0);
    }

    impl CLI{
        pub fn new() -> Self{
            Self { dbg: false, method: "RECEIVE".to_string(), send: HashMap::new(), receive: vec![], srcdir:None,connection:Connection::default(),config:None}
        }


        pub fn Parse_Args(&mut self){
            let args = std::env::args().skip(1).collect::<Vec<String>>();
            if args.is_empty() {
                Help();
            }
            self.method = (&args[0]).to_ascii_uppercase();
            let args = args.into_iter().skip(1).collect::<Vec<String>>();
            for i in &args{
                if matches!(&i[..],"-d" | "-D" | "--debug" | "--Debug"){
                    self.dbg = true;
                }else if i.starts_with("--src=") || i.starts_with("-s="){
                    self.srcdir = Some(PathBuf::from(&i[i.find("=").unwrap()+1..]));
                }else if i.starts_with("--config=") || i.starts_with("-conf="){
                    self.config = Some(i[i.find("=").unwrap()+1..].to_string());
                }else if i.starts_with("--conn=") || i.starts_with("-c="){
                    let (sp,ed) = (i.find("(").unwrap()+1,i.find(")").unwrap());
                    let fields = i[sp..ed].split(",").map(|x| x.trim()).collect::<Vec<&str>>();
                    self.connection.url = fields[0].to_string();
                    self.connection.ip = fields[1].to_string();
                    self.connection.port = fields[2].parse().expect("Port is a u32");
                    self.connection.hostname = fields[3].to_string();
                }else if i.starts_with("--send"){
                    let (sp,ed) = (i.find("(").unwrap()+1,i.find(")").unwrap());
                    let recipient = &i[i.find("=").unwrap()+1..sp-1];
                    let data = i[sp..ed].split(",").map(|x| PathBuf::from(x.trim())).collect::<Vec<PathBuf>>();
                    self.send.insert(recipient.to_string(),data);
                }else if i.starts_with("--receive"){
                    let (sp,ed) = (i.find("(").unwrap()+1,i.find(")").unwrap());
                    self.receive.append(&mut i[sp..ed].split(",").map(|x| x.trim().to_string()).collect::<Vec<String>>());
                }else{
                    Help();
                }



            }

        }

        



    }


    #[derive(Debug,Clone)]
    pub struct Connection{
        pub url:String,
        pub ip: String,
        pub port: u32,
        pub hostname:String,
    }

    impl Default for Connection{
        fn default() -> Self {
            Self { url: URL.to_string(), ip: IP.to_string(), port: PORT, hostname: HOSTNAME.to_string()}
        }
    }




}