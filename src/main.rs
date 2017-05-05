extern crate chrono;
extern crate serde;
extern crate serde_yaml;
#[macro_use]
extern crate serde_derive;

use std::process::Command;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use chrono::Local;

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Config{
 pub host_path: String,
 pub host_name_prefix: String,
 pub ssh_command: String,
 pub run_command: String 		
}

fn main(){
	let config: Result<Config,serde_yaml::Error> = parse_config();
	match config{
		Ok(config)=>{
			let designations: Vec<String> = get_designations((&config.host_path).to_string(),(&config.host_name_prefix).to_string());
			let desigs = designations.clone();
			let commands: Vec<String> = get_call_commands(designations,config);
			let now = Local::now();
			let mut success_trains = File::create(format!("added_to_trains_{}.log",now.format("%Y-%m-%d][%H:%M:%S"))).unwrap();
			let mut error_trains = File::create(format!("error_in_trains_{}.log",now.format("%Y-%m-%d][%H:%M:%S"))).unwrap();
			for (index,command) in commands.iter().enumerate(){
			//println!("Command {}",command);
			let desig = &desigs[index];
			let response=call_command(command.to_string(),desig.to_string());
			match response{
			Ok(z)=>{
			 success_trains.write(format!("{}\n",z).as_bytes()).unwrap();     
			 println!("Result {}",z.to_string());
			},
			Err(err)=>{
				error_trains.write(format!("{}\n",err).as_bytes()).unwrap();
				//println!("No result! {}",err);
			}
			}
			}
		},
		Err(err)=>{
			println!("{}",err);
			panic!("No config file found!!");	
		}
	}
        
}

///Parse configurations
fn parse_config() -> Result<Config,serde_yaml::Error>{
	//let conf = Config{host_path:"/fdsfs".to_string(),host_name_prefix: "rita".to_string()};
	//let s = serde_yaml::to_string(&conf).unwrap();	
	//println!("Test:::::::{:?}",s);

	let mut yaml_file = File::open("../resources/config.yaml").unwrap();
	let mut yaml_data = String::new();
	yaml_file.read_to_string(&mut yaml_data).unwrap();
	let config: Result<Config,serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
	match config{
		Ok(config)=>{return Ok(config);},
		Err(e)=>{println!("{}",e); return Err(e);}
	}
}

///Prepare rsync commands for all designations
fn get_call_commands(desigs: Vec<String>,config: Config) -> Vec<String>{
	let mut commands:Vec<String>=Vec::new();
        let command_part1 = config.ssh_command.to_string();
	let command_part2 = config.run_command.to_string();
	//let command_part2 = " rsync --password-file=/var/ND/rita/config/rsync_pwd -aPv --exclude='rsync_pwd' --exclude='.pyc' --exclude='*.out' rsync://ns_user@172.16.211.20/RITA-SYNC/logs/NS_UPDATE/rita/* /var/ND/rita/ && rsync --password-file=/var/ND/rita/config/rsync_pwd -aPv --exclude='rsync_pwd' --exclude='.pyc' --exclude='*.out' rsync://ns_user@172.16.211.20/RITA-SYNC/logs/NS_UPDATE/ndtv/* /var/ND/ndtv/";
	for desig in desigs{
		commands.push(format!("{} {} {}",command_part1,desig,command_part2));		
	}
	return commands;	
}	

///Fetch the rita designations from host file
fn get_designations(file_path: String,prefix: String) -> Vec<String>{
	let file_content = File::open(&file_path);
	let mut desigs: Vec<String> = Vec::new();
	match file_content{
	Ok(file)=> {
		let reader = BufReader::new(file);
        	for line in reader.lines(){
            		match line{
                	Ok(output)=>{
                  		if output.starts_with(&prefix){
                        		desigs.push(output[5..].trim().to_string());
                  		}
               		 },
                	Err(err)=>{println!("{}",err);}
            		}
        	} 
	},			
	Err(err)=>{
		println!("{}",err);
	}	
	}
	return desigs;
}

///Call command
fn call_command(cmd: String,desig: String)-> Result<String,String>{
	let parts = split_command(cmd);
	let mut main_cmd = "";
	let mut args: Vec<String> = Vec::new();
	for (index,item) in parts.iter().enumerate(){
	   if index == 0{
		main_cmd=item;
	   }
           else{
		args.push(item.to_string());
	   }
	}
	let mut process = Command::new(main_cmd);
	for arg in args{
	   process.arg(arg);
	}
	let result = process.output();
        match result{
        Ok(output)=> {
        println!("Successful {} Command :: {:?}",String::from_utf8_lossy(&output.stdout),process);
		println!("Status {:?}",output.status.success());
		let result=String::from_utf8_lossy(&output.stdout);
		println!("{}",result);
		let err=String::from_utf8_lossy(&output.stderr);
		if output.status.success(){
			return Ok(desig);
		}	
		else{
			return Err(format!("{}{}{}",desig,err,result.to_string()));
		}
        },
        Err(e)=>{
                println!("Error {} Command:: {:?}",e.to_string(),process);
		return Err("Unable to execute command".to_string());
         }
        }
}	     
       


///Split command from String to Vector
fn split_command(cmd: String) -> Vec<String>{
	let command_parts: Vec<&str> = cmd.split(" ").collect();
	let parts: Vec<String>=command_parts.into_iter().map(|s| s.to_string()).collect();
	return parts;
}
