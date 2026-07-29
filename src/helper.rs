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
        pub recieve: Vec<String>,
        pub connection:Connection,
        pub srcdir: Option<PathBuf>,
        pub config:Option<String>,
    }

    pub fn Help(){
        println!("{DBG_STR}");
        exit(0);

    }

    impl CLI{
        pub fn new() -> Self{
            Self { dbg: false, method: "RECIEVE".to_string(), send: HashMap::new(), recieve: vec![], srcdir:None,connection:Connection::default(),config:None}
        }


        pub fn Parse_Args(&mut self){
            let args = std::env::args().skip(1).collect::<Vec<String>>();
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
                }else if i.starts_with("--recieve"){
                    let (sp,ed) = (i.find("(").unwrap()+1,i.find(")").unwrap());
                    self.recieve.append(&mut i[sp..ed].split(",").map(|x| x.trim().to_string()).collect::<Vec<String>>());
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