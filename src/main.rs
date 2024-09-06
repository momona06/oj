use std::{collections::HashMap, fs::{File, self}, process::Stdio, time::{Duration, Instant}, io::{BufReader, BufRead}};
use actix_web::{get, middleware::Logger, post, web, App, HttpServer, Responder, HttpResponse, http, put};
use env_logger;
use log;
use serde::{Deserialize, Serialize};
use std::sync::{Arc,Mutex};
use lazy_static::lazy_static;
use http::StatusCode;
use serde_derive::*;
use serde_json::*;
use clap::*;
use std::process::Command;
use chrono::*;
use wait_timeout::ChildExt;

lazy_static! {
    pub static ref JOB_LIST: Arc<Mutex<Vec<PostResponseJob>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref USER_LIST: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref CONTEST_LIST: Arc<Mutex<Vec<Contest>>>=Arc::new(Mutex::new(Vec::new()));
    pub static ref FILE_DATA: Arc<Mutex<FileData>>=Arc::new(Mutex::new(FileData { jobs: Vec::new(), users: Vec::new(), contests: Vec::new() }));
}
// the static variables that will be needed

fn min <T: PartialOrd> (x: T, y: T) -> T {
    if x > y {
        y
    }
    else {
        x
    }
}
// fn: find the min variable;
// x & y: the variable that need to be compared;
// out: the min variable

fn cmp_date(job_time:NaiveDateTime, info_time:NaiveDateTime, job_timenano: i32, info_timenano:i32) -> bool {
    if job_time.year() < info_time.year() {
        return false;
    }
    else if job_time.year() == info_time.year() {
        if job_time.month() < info_time.month() {
            return false;
        }
        else if job_time.month() == info_time.month() {
            if job_time.day() < info_time.day() {
                return false;
            }
            else if job_time.day() == info_time.day() {
                if job_time.hour() < info_time.hour() {
                    return false;
                }
                else if  job_time.hour() == info_time.hour() {
                    if job_time.minute() < info_time.minute() {
                        return false;
                    }
                    else if job_time.minute() == info_time.minute() {
                        if job_time.second() < info_time.second() {
                            return false;
                        }                  
                        else if job_time.second() == info_time.second() {
                            if job_timenano < info_timenano {
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
    return true;

}
// fn: compare two datetime;
// job_time, job_timenano & info_time, info_timenano: the NaiveDateTime and the nanosecond of two moment;
// if job is later then print true else false

fn time_reprint(timec: DateTime<Utc>, mut timec_str: String, timec_nano: u32) -> String {
    
    if timec.month() < 10 && timec.day() < 10 {
        timec_str = format!("{}-0{}-0{}T", timec.year(), timec.month(), timec.day() );
    }
    else if timec.month() < 10 && timec.day() >= 10 {       
        timec_str = format!("{}-0{}-{}T", timec.year(), timec.month(), timec.day() );
    }
    else if timec.month() >= 10 && timec.day() < 10 {
        timec_str = format!("{}-{}-0{}T", timec.year(), timec.month(), timec.day() );
    }
    else {   
        timec_str = format!("{}-{}-{}T", timec.year(), timec.month(), timec.day());
    }

    if timec.hour() < 10 {
        timec_str = format!("{}0{}", timec_str.clone(), timec.hour().to_string());
    }
    else {
        timec_str = format!("{}{}", timec_str.clone(), timec.hour().to_string());
    }

    if timec.minute() < 10 && timec.second() < 10 {
        timec_str = format!("{}:0{}:0{}.{}Z",timec_str.clone(),
        timec.minute().to_string(), timec.second().to_string(), timec_nano.to_string() );
    }
    else if timec.minute() < 10 && timec.second() >= 10 {       
        timec_str = format!("{}:0{}:{}.{}Z", timec_str.clone(),
        timec.minute().to_string(), timec.second().to_string(), timec_nano.to_string() );
    }
    else if timec.minute() >= 10 && timec.second() < 10 {
        timec_str = format!("{}:{}:0{}.{}Z", timec_str.clone(),
        timec.minute().to_string(), timec.second().to_string(), timec_nano.to_string() );
    }
    else {   
        timec_str = format!("{}:{}:{}.{}Z", timec_str.clone(),
        timec.minute().to_string(), timec.second().to_string(), timec_nano.to_string() );
    }

    timec_str.clone()
}
// fn: reorganize the format of the data string
// timec: the DateTime, timec_str: the string needed, timec_nano: the nanosecond;
// the string needed

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileData {
    jobs: Vec<PostResponseJob>,
    users: Vec<User>,
    contests: Vec<Contest>,

}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    id: Option<i32>,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    server: Bind,
    problems: Vec<Problem>,
    languages: Vec<Language>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bind {
    bind_address: String,
    bind_port: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Problem {
    id: i32,
    name: String,
    #[serde(rename="type")]
    typ: ProblemType,
    misc: Option<Misc>,
    cases: Vec<Case>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Misc {
    packing: Option<Vec<Vec<i32>>>,
    special_judge: Option<Vec<String>>,
    dynamic_ranking_ratio: Option<f64>,

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProblemType {
    standard,
    strict,
    spj,
    dynamic_ranking,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Case {
    score: f64,
    input_file: String,
    answer_file: String,
    time_limit: i32,
    memory_limit: i32,
}//case in config

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    name: String,
    file_name: String,
    command: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum State {
    Queueing,
    Running,
    Finished,
    Canceled,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Result {
    Waiting,
    Running,
    Accepted,
    #[serde(rename = "Compilation Error")]
    CompilationError,
    #[serde(rename = "Compilation Success")]
    CompilationSuccess,
    #[serde(rename = "Wrong Answer")]
    WrongAnswer,
    #[serde(rename = "Runtime Error")]
    RuntimeError,
    #[serde(rename = "Time Limit Exceeded")]
    TimeLimitExceeded,
    #[serde(rename = "Memory Limit Exceeded")]
    MemoryLimitExceeded,
    #[serde(rename = "System Error")]
    SystemError,
    #[serde(rename = "SPJ Error")]
    SPJError,
    Skipped,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    code: i32,
    reason: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostCase {
    id: i32,
    result: Result,
    time: u128,
    memory: i32,
    info: String,
}//case in post response

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostInJob {
    source_code: String,
    language: String,
    user_id: i32,
    contest_id: i32,
    problem_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostResponseJob {
    id: i32,
    create_time: String,
    updated_time: String,
    submission: PostInJob,
    state: State,
    result: Result,
    score: f64,
    cases: Vec<PostCase>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUrl {
    user_id: Option<i32>,
    user_name: Option<String>,
    contest_id: Option<i32>,
    problem_id: Option<i32>,
    language: Option<String>,
    from: Option<String>,
    to: Option<String>,
    state: Option<State>,
    result: Option<Result>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContestRank {
    scoring_rule: Option<SRule>,
    tie_breaker: Option<TBreaker>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SRule {
    latest,
    highest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TBreaker {
    submission_time,
    submission_count,
    user_id,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contest {
    id: Option<i32>,
    name: String,
    from: String,
    to: String,
    problem_ids:Vec<i32>,
    user_ids:Vec<i32>,
    submission_limit: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserRank {
    user: User,
    rank: i32,
    scores: Vec<f64>,
}
// the structs and enums that the oj needs


#[post("/jobs")]
async fn post_jobs(body: web::Json<PostInJob>, config: web::Data<Config>) -> impl Responder {

    let mut postresponsejob: PostResponseJob = PostResponseJob { id: 0, create_time: String::new(), updated_time: String::new(),
         submission: body.clone(), state: State::Queueing, result: Result::Waiting, score: 0.0, cases: Vec::new() };
        postresponsejob.cases.push(PostCase { id: 0, result: Result::Waiting, time: 0, memory: 0, info: String::new() });
    // the init of postresponsejob

    let timec: DateTime<Utc> = Utc::now();
    let mut timec_nano = timec.nanosecond();
    while timec_nano / 1000 > 0 {
        timec_nano = timec_nano / 10;
    }
    let timec_str0 = String::new();

    let timec_str = time_reprint(timec, timec_str0, timec_nano);

    postresponsejob.create_time = timec_str.clone();
    // the push of the create_time

    let mut langtest: bool = false;
    let mut lang_num = 0;
    for i in 0..config.languages.len() {
        if config.languages[i].name == body.language {
            langtest = true;
            lang_num = i;
            break;
        }
    }
    // confirming the existence of the language

    match langtest {
        true => {
            let mut probtest: bool = false;
            let mut prob_num = 0;
            for i in 0..config.problems.len() {
                if config.problems[i].id == body.problem_id {
                    probtest = true;
                    prob_num = i;
                    break;
                }
            }

            //confirming the existence of the problem

            match probtest {
                true => {          
                    let user_list = USER_LIST.lock().unwrap();

                    if body.user_id > (user_list.len() as i32) - 1 {
                        return HttpResponse::NotFound()
                        .status(StatusCode::from_u16(404 as u16).unwrap())
                        .json(Error {
                            code: 3,
                            reason: "ERR_NOT_FOUND".to_string(),
                            message: String::new(),
                        });
                    }
                    //confirming the existence of the user

                    let contest_list = CONTEST_LIST.lock().unwrap(); 

                    if body.contest_id > (contest_list.len() as i32) - 1 {
                        return HttpResponse::NotFound()
                        .status(StatusCode::from_u16(404 as u16).unwrap())
                        .json(Error {
                            code: 3,
                            reason: "ERR_NOT_FOUND".to_string(),
                            message: String::new(),
                        });
                    }
                    //confirming the existence of the contest

                    else {
                        if body.contest_id != 0 {
                        let mut out = false;
                        let mut user_e = false;
                        let mut prob_e = false;
                        let mut outline = true;

                        for i in &contest_list[body.contest_id as usize].user_ids {
                            if i == &body.user_id {
                                user_e = true;
                                break;
                            }
                        }
                        if user_e == false {
                            out = true;
                        }

                        for i in &contest_list[body.contest_id as usize].problem_ids {
                            if i == &body.problem_id {
                                prob_e = true;
                                break;
                            }
                        }
                        if prob_e == false {
                            out = true;
                        }

                        let job_list = JOB_LIST.lock().unwrap();
                        let job_list_num = job_list.len();
                        let mut count = 0;
                        for i in 0..job_list_num {
                            if job_list[i].submission.problem_id == body.problem_id && job_list[i].submission.user_id == body.user_id
                            && job_list[i].submission.contest_id == body.contest_id {
                                count += 1;
                            }
                        }
                        drop(job_list);

                        if count >= contest_list[body.contest_id as usize].submission_limit && contest_list[body.contest_id as usize].submission_limit != 0 {
                            outline = false;
                        }

                        
                        let from = NaiveDateTime::parse_from_str(&contest_list[body.contest_id as usize].from, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                        let from_nano = contest_list[body.contest_id as usize].from[20..23].parse::<i32>().unwrap();     
                        
                                       
                        let to = NaiveDateTime::parse_from_str(&contest_list[body.contest_id as usize].to, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                        let to_nano = contest_list[body.contest_id as usize].to[20..23].parse::<i32>().unwrap();                             

               
                        let create = NaiveDateTime::parse_from_str(&postresponsejob.create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                        let create_nano = postresponsejob.create_time[20..23].parse::<i32>().unwrap();       
                        
                        if !cmp_date(create, from, create_nano, from_nano) {
                            out = true;
                        }


                        if !cmp_date(to, create, to_nano, create_nano) {
                            out = true;
                        }


                        if out == true {
                            return HttpResponse::BadRequest()
                            .status(StatusCode::from_u16(400 as u16).unwrap())
                            .json(Error {
                                code: 1,
                                reason: "ERR_INVALID_ARGUMENT".to_string(),
                                message: String::new(),
                            });
                        }

                        if outline == false {
                            return HttpResponse::BadRequest()
                            .status(StatusCode::from_u16(400 as u16).unwrap())
                            .json(Error {
                                code: 4,
                                reason: "ERR_RATE_LIMIT".to_string(),
                                message: String::new(),
                            });
                        }
                    }

                    }//check whether the contest is legal or not 
                                     

                    for i in 0..config.problems[prob_num].cases.len() {
                        postresponsejob.cases.push( PostCase { id: i as i32+1, result: Result::Waiting, time: 0, memory: 0, info: String::new() });
                    }//fill the response cases

                    let source_code = body.source_code.clone();

                    std::fs::create_dir("./TMPDIR");

                    let file_before = "TMPDIR/".to_string();
                    let source_file = format!("{}{}",file_before,config.languages[lang_num].file_name.clone());
                    let exe_file = format!("{}{}",file_before,"test.exe".to_string());
                    let out_file = format!("{}{}",file_before,"test.out".to_string());

                    File::create(&source_file);
                    File::create(&exe_file);
                    fs::write(&source_file,source_code);

                    let mut command:Vec<String> = config.languages[lang_num].command.clone();
                    for i in &mut command {
                        if i == "%INPUT%" {
                            *i = source_file.clone();
                        }
                        else if i == "%OUTPUT%" {
                            *i = exe_file.clone();
                        }
                    }

                    postresponsejob.state = State::Running;
                    postresponsejob.result = Result::Running;
                    postresponsejob.cases[0].result = Result::Running;

                    let complie_cmd = Command::new(&command[0])
                                            .args(&command[1..])
                                            .status()
                                            .expect("Complication Error");
                
                    // do the compiling task
           

                    if !complie_cmd.success() {

                        postresponsejob.state = State::Finished;
                        postresponsejob.result = Result::CompilationError;
                        postresponsejob.cases[0].result = Result::CompilationError;

                        let timec: DateTime<Utc> = Utc::now();
                        let mut timec_nano = timec.nanosecond();
                        while timec_nano / 1000 > 0 {
                            timec_nano = timec_nano / 10;
                        }
                        let timec_str0 = String::new();
                    
                        let timec_str = time_reprint(timec, timec_str0, timec_nano);
                    
                        postresponsejob.updated_time = timec_str.clone();
                        
                        return HttpResponse::Ok()
                                .status(StatusCode::from_u16(200 as u16).unwrap())
                                .json(postresponsejob);         

                    }
                                 
                    postresponsejob.cases[0].result = Result::CompilationSuccess;

                    // check the process of the compiling

                    let mut score: f64 = 0.0;

                    for i in 0..config.problems[prob_num].cases.len() {

                        let out_file_f = File::create(&out_file).unwrap();
                        let in_file_f = File::open(&config.problems[prob_num].cases[i].input_file).unwrap();
                        let test_now = Instant::now();
                        let mut time_exceed: bool = false;  

                        if config.problems[prob_num].cases[i].time_limit != 0 {

                            let usec = Duration::from_micros(config.problems[prob_num].cases[i].time_limit as u64);

                            let test_cmd = Command::new(&exe_file)
                                .stdin(Stdio::from(in_file_f))
                                .stdout(Stdio::from(out_file_f))
                                .spawn()
                                .unwrap()
                                .wait_timeout(usec)
                                .unwrap();

                            // execute & print the code

                            time_exceed = match test_cmd {
                                    Some(_) => {
                                        false
                                    }
                                    None => {
                                        true
                                    }
                            };
                        }

                        else {
                            let test_cmd = Command::new(&exe_file)
                                    .stdin(Stdio::from(in_file_f))
                                    .stdout(Stdio::from(out_file_f))
                                    .spawn()
                                    .unwrap();
                                   
                        }

                        let test_time = test_now.elapsed().as_micros();
                        postresponsejob.cases[i+1].time = test_time;


                        let mut run_access: bool = true;

                        if time_exceed == false {

                            let out_file_f = File::create(&out_file).unwrap();

                            let in_file_f = File::open(&config.problems[prob_num].cases[i].input_file).unwrap();
                                    
                            let mut test_cmd_t = Command::new(&exe_file)
                            .stdin(Stdio::from(in_file_f))
                            .stdout(Stdio::from(out_file_f))
                            .spawn()
                            .unwrap();
                                                    
                            run_access = test_cmd_t.wait().unwrap().success();

                            if run_access == false {
                                postresponsejob.cases[i+1].result = Result::RuntimeError;
                                match postresponsejob.result {
                                    Result::Running => {
                                        postresponsejob.result = Result::RuntimeError;
                                    }
                                    _ => {                                          
                                    }
                                }                           
                            }                           
                        }
                                             
                        // check whether there is a runtime error
                        

                        if time_exceed == true {
                            postresponsejob.cases[i+1].result = Result::TimeLimitExceeded;
                            match postresponsejob.result {
                                Result::Running => {
                                    postresponsejob.result = Result::TimeLimitExceeded;
                                }
                                _ => {                                          
                                }
                            }
                        }
                    

                    if time_exceed == false && run_access == true {
                        match config.problems[prob_num].typ {

                            ProblemType::standard => {

                                let ans_file_buf = File::open(&config.problems[prob_num].cases[i].answer_file).unwrap();
                                let out_file_buf = File::open(&out_file).unwrap();
                                let ans_buf = BufReader::new(ans_file_buf);
                                let out_buf = BufReader::new(out_file_buf);
                                let mut ans_vec: Vec<String> = Vec::new();
                                let mut out_vec: Vec<String> = Vec::new();

                                for line in ans_buf.lines() {
                                    ans_vec.push(line.unwrap_or_default());
                                }

                                for line in out_buf.lines() {
                                    out_vec.push(line.unwrap_or_default());                         
                                }

                                let mut res: bool = true;

                                for i in 0..min::<usize>(ans_vec.len(),out_vec.len()) {
                                    if ans_vec[i].trim_end() != out_vec[i].trim_end() {
                                        res = false;
                                        break;
                                    }
                                }

                                if ans_vec.len() > out_vec.len() {
                                    for i in out_vec.len()..ans_vec.len() {
                                        if ans_vec[i].trim_end().len() != 0 {
                                            res = false;
                                            break;
                                        }
                                    }
                                }

                                else if  ans_vec.len() < out_vec.len() {
                                    for i in ans_vec.len()..out_vec.len() {
                                        if out_vec[i].trim_end().len() != 0 {
                                            res = false;
                                            break;
                                        }
                                    }
                                }

                                //compare the files line by line
                                
                                if let Some(c) = config.problems[prob_num].misc.clone() {

                                    if let Some(p) = c.packing.clone() {

                                        if res == true && postresponsejob.cases[i+1].result != Result::Skipped {
                                            postresponsejob.cases[i+1].result = Result::Accepted;
                                            score += config.problems[prob_num].cases[i].score;      
                                        }
                                        else if res == true && postresponsejob.cases[i+1].result == Result::Skipped {                                     
                                        }
                                      
                                        else if res == false &&postresponsejob.cases[i+1].result == Result::Skipped {
                                        }    

                                        else {
                                            for pack1 in p {
                                                let mut pack1_conf = false;
                                                for pack2 in &pack1 {
                                                    if pack2 == &((i as i32) + 1) {
                                                        pack1_conf = true;
                                                        break;
                                                    }
                                                }
                                                if pack1_conf == true {
                                                    for pack2 in pack1 {
                                                        postresponsejob.cases[pack2 as usize].result = Result::Skipped;
                                                    }
                                                }
                                            }
                                            postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                            match postresponsejob.result {
                                                Result::Running => {
                                                    postresponsejob.result = Result::WrongAnswer;
                                                }
                                                _ => {                                          
                                                }
                                            } 
                                        }
                                    }

                                    else {

                                        if res == true {
    
                                            postresponsejob.cases[i+1].result = Result::Accepted;
                                            score += config.problems[prob_num].cases[i].score;
                                        }
                                    
                                        else {                                                                    
                                            postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                            match postresponsejob.result {
                                                Result::Running => {
                                                    postresponsejob.result = Result::WrongAnswer;
                                                }
                                                _ => {                                          
                                                }
                                            }                            
                                        }
                                    }
                                }     

                                else {
                                    if res == true {
    
                                        postresponsejob.cases[i+1].result = Result::Accepted;
                                        score += config.problems[prob_num].cases[i].score;
                                    }
                                
                                    else {                                                                    
                                        postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                        match postresponsejob.result {
                                            Result::Running => {
                                                postresponsejob.result = Result::WrongAnswer;
                                            }
                                            _ => {                                          
                                            }
                                        }                            
                                    }
                                }                                                                               
                            }

                            ProblemType::strict => {
                                
                                let ans_str: Vec<u8> = std::fs::read(&config.problems[prob_num].cases[i].answer_file).unwrap();
                                let out_str: Vec<u8> = std::fs::read(&out_file).unwrap();

                                let res: bool = ans_str == out_str;
    
                                if let Some(c) = config.problems[prob_num].misc.clone() {

                                    if let Some(p) = c.packing.clone() {

                                        if res == true && postresponsejob.cases[i+1].result != Result::Skipped {
                                            postresponsejob.cases[i+1].result = Result::Accepted;
                                            score += config.problems[prob_num].cases[i].score;      
                                        }
                                        else if res == true && postresponsejob.cases[i+1].result == Result::Skipped {                                     
                                        }
                                      
                                        else if res == false &&postresponsejob.cases[i+1].result == Result::Skipped {
                                        }    

                                        else {
                                            for pack1 in p {
                                                let mut pack1_conf = false;
                                                for pack2 in &pack1 {
                                                    if pack2 == &((i as i32) + 1) {
                                                        pack1_conf = true;
                                                        break;
                                                    }
                                                }
                                                if pack1_conf == true {
                                                    for pack2 in pack1 {
                                                        postresponsejob.cases[pack2 as usize].result = Result::Skipped;
                                                    }
                                                }
                                            }
                                            postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                            match postresponsejob.result {
                                                Result::Running => {
                                                    postresponsejob.result = Result::WrongAnswer;
                                                }
                                                _ => {                                          
                                                }
                                            } 
                                        }
                                    }

                                    else {

                                        if res == true {
                                            postresponsejob.cases[i+1].result = Result::Accepted;
                                            score += config.problems[prob_num].cases[i].score;
                                        }
                                    
                                        else {                                                                    
                                            postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                            match postresponsejob.result {
                                                Result::Running => {
                                                    postresponsejob.result = Result::WrongAnswer;
                                                }
                                                _ => {                                          
                                                }
                                            }                            
                                        }
                                    }
                                }     

                                else {
                                    if res == true {
    
                                        postresponsejob.cases[i+1].result = Result::Accepted;
                                        score += config.problems[prob_num].cases[i].score;
                                    }
                                
                                    else {                                                                    
                                        postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                        match postresponsejob.result {
                                            Result::Running => {
                                                postresponsejob.result = Result::WrongAnswer;
                                            }
                                            _ => {                                          
                                            }
                                        }                            
                                    }
                                }                 
                            }

                            ProblemType::dynamic_ranking => {

                                if let Some(c) = config.problems[prob_num].misc.clone() {
                                    if let Some(dyn_r) = c.dynamic_ranking_ratio {

                                        
                                let ans_file_buf = File::open(&config.problems[prob_num].cases[i].answer_file).unwrap();
                                let out_file_buf = File::open(&out_file).unwrap();
                                let ans_buf = BufReader::new(ans_file_buf);
                                let out_buf = BufReader::new(out_file_buf);
                                let mut ans_vec: Vec<String> = Vec::new();
                                let mut out_vec: Vec<String> = Vec::new();


                                for line in ans_buf.lines() {
                                    ans_vec.push(line.unwrap_or_default());
                                }

                                for line in out_buf.lines() {
                                    out_vec.push(line.unwrap_or_default());                         
                                }

                                let mut res: bool = true;

                                for i in 0..min::<usize>(ans_vec.len(),out_vec.len()) {
                                    if ans_vec[i].trim_end() != out_vec[i].trim_end() {
                                        res = false;
                                        break;
                                    }
                                }

                                if ans_vec.len() > out_vec.len() {
                                    for i in out_vec.len()..ans_vec.len() {
                                        if ans_vec[i].trim_end().len() != 0 {
                                            res = false;
                                            break;
                                        }
                                    }
                                }

                                else if  ans_vec.len() < out_vec.len() {
                                    for i in ans_vec.len()..out_vec.len() {
                                        if out_vec[i].trim_end().len() != 0 {
                                            res = false;
                                            break;
                                        }
                                    }
                                }
                                // compare the files line by line

                                if res == true { 
                                    postresponsejob.cases[i+1].result = Result::Accepted;
                                    score += config.problems[prob_num].cases[i].score * (1.0 - dyn_r);
                                }
                            
                                else {                                                                    
                                    postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                    match postresponsejob.result {
                                        Result::Running => {
                                            postresponsejob.result = Result::WrongAnswer;
                                        }
                                        _ => {                                          
                                        }
                                    }                            
                                }
                                    }
                                }
                                // giving the score and the state

                            }

                            ProblemType::spj => {

                                if let Some(c) = config.problems[prob_num].misc.clone() {
                                    if let Some(sp_j) = c.special_judge {

                                        let mut command_spj:Vec<String> = sp_j.clone();

                                        for j in &mut command_spj {
                                            if j == "%ANSWER%" {
                                                *j = config.problems[prob_num].cases[i].answer_file.clone();
                                            }
                                            else if j == "%OUTPUT%" {
                                                *j = "./TMPDIR/test.out".to_string();
                                            }
                                        }

                                        let in_time = File::create("./TMPDIR/in_time.in").unwrap();
                                        let out_time = File::create("./TMPDIR/out_time.out").unwrap();

                                        let spj_cmd = Command::new(&command_spj[0])
                                                                .args(&command_spj[1..])
                                                                .stdin(Stdio::from(in_time))
                                                                .stdout(Stdio::from(out_time))
                                                                .status()
                                                                .unwrap();
                                     
                                        if !spj_cmd.success() {
                                            postresponsejob.cases[i+1].result = Result::SPJError;

                                            match postresponsejob.result {
                                                Result::Running => {
                                                postresponsejob.result = Result::SPJError;
                                                }
                                                _ => {                                          
                                                }
                                            }
                                        }

                                        //compile and run the code
                                        let out_time_open = File::open("./TMPDIR/out_time.out").unwrap();
                                        let out_buf_time = BufReader::new(out_time_open);                                      
                                        let mut out_vec_time: Vec<String> = Vec::new();
      
                                        for line in out_buf_time.lines() {
                                            out_vec_time.push(line.unwrap_or_default());                         
                                        }

                                        
                                        if out_vec_time.len() != 2 {
                                            postresponsejob.cases[i+1].result = Result::SPJError;

                                            match postresponsejob.result {
                                                Result::Running => {
                                                postresponsejob.result = Result::SPJError;
                                                }
                                                _ => {                                          
                                                }
                                            }
                                        }

                                        else {
                                        if out_vec_time[0] == "Accepted".to_string() {

                                            postresponsejob.cases[i+1].result = Result::Accepted;
                                            score += config.problems[prob_num].cases[i].score;
                                            postresponsejob.cases[i+1].info= out_vec_time[1].clone();
                                            
                                        }

                                        else if out_vec_time[0] == "Wrong Answer".to_string() {
                                            postresponsejob.cases[i+1].result = Result::WrongAnswer;

                                            match postresponsejob.result {
                                                Result::Running => {
                                                postresponsejob.result = Result::WrongAnswer;
                                                }
                                                _ => {                                          
                                                }
                                            }
                                            postresponsejob.cases[i+1].info= out_vec_time[1].clone();
                                        }

                                        else {

                                            postresponsejob.cases[i+1].result = Result::SPJError;

                                            match postresponsejob.result {
                                                Result::Running => {
                                                postresponsejob.result = Result::SPJError;
                                                }
                                                _ => {                                          
                                                }
                                            }
                                        }
                                    }

                                        // process the result and push to the struct
                                    }
                                }
                            }
                        }
                    }

                    // compare the out and the ans files
                }
                    
                    std::fs::remove_dir_all("./TMPDIR").unwrap();
                    
                    let timec: DateTime<Utc> = Utc::now();
                    let mut timec_nano = timec.nanosecond();
                    while timec_nano / 1000 > 0 {
                        timec_nano = timec_nano / 10;
                    }
                                 
                    let timec_str0 = String::new();

                    let timec_str = time_reprint(timec, timec_str0, timec_nano);
          
                    postresponsejob.updated_time = timec_str.clone();
                    
                    // push the update_time

                    postresponsejob.state = State::Finished;
                    postresponsejob.score = score;

                    let mut job_list = JOB_LIST.lock().unwrap();

                    match postresponsejob.result {
                        Result::Running => {
                            postresponsejob.result = Result::Accepted;
                        }
                        _ => {
                        }
                    }

                    postresponsejob.id = job_list.len() as i32;
                    job_list.push(postresponsejob.clone());

                    let mut file_data = FILE_DATA.lock().unwrap();
                    file_data.jobs = job_list.clone();
                    fs::remove_file("./datafile").unwrap();
                    let mut file = std::fs::File::create("./datafile").unwrap();
                    let data_time = file_data.clone();
                    serde_json::to_writer(&mut file, &data_time).unwrap();
                                        
                    drop(job_list);
                    drop(file_data);

                    // finish the persistent storage function

                    return HttpResponse::Ok()
                    .status(StatusCode::from_u16(200).unwrap())
                    .json(postresponsejob);


                }

                false => {
                    return HttpResponse::BadRequest()
                    .status(StatusCode::from_u16(404 as u16).unwrap())
                    .json(Error {
                        code: 3,
                        reason: "ERR_NOT_FOUND".to_string(),
                        message: String::new(),
                    });
                }
            }
        }

        false => {
            return HttpResponse::BadRequest()
            .status(StatusCode::from_u16(404 as u16).unwrap())
            .json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: String::new(),
            });
        }
    }


}
// fn: post a job to the oj;
// body: the postin http json struct, config: the configration file;
// the response of the http request 


#[get("/jobs")]
async fn get_jobs(web::Query(info):web::Query<GetUrl>) -> impl Responder {
    let mut select: Vec<bool> = Vec::new();

    let job_list = JOB_LIST.lock().unwrap();
    let user_list = USER_LIST.lock().unwrap();


    let job_num = job_list.len(); 
    for _ in 0..job_num {
        select.push(true);
    }

    if let Some(c) = info.language {
        for i in 0..job_num {
            if job_list[i].submission.language != c {
                select[i] = false;
            }
        }
    }

    if let Some(c) = info.from {
        let info_time = NaiveDateTime::parse_from_str(&c, "%Y-%m-%dT%H:%M:%S.3fZ").unwrap();
        for i in 0..job_num {
            let job_time = NaiveDateTime::parse_from_str(&job_list[i].create_time, "%Y-%m-%dT%H:%M:%S.3fZ").unwrap();
            if job_time.year() < info_time.year() {
                select[i] = false;
            }
            else if job_time.year() == info_time.year() {
                if job_time.month() < info_time.month() {
                    select[i] = false;
                }
                else if job_time.month() == info_time.month() {
                    if job_time.day() < info_time.day() {
                        select[i] = false;
                    }
                    else if job_time.day() == info_time.day() {
                        if job_time.hour() < info_time.hour() {
                            select[i] = false;
                        }
                        else if  job_time.hour() == info_time.hour() {
                            if job_time.minute() < info_time.minute() {
                                select[i] = false;
                            }
                            else if job_time.minute() == info_time.minute() {
                                if job_time.second() < info_time.second() {
                                    select[i] = false;
                                }                               
                            }
                        }
                    }
                }
            }
        }
    }
    
    if let Some(c) = info.to {
        let info_time = NaiveDateTime::parse_from_str(&c, "%Y-%m-%dT%H:%M:%S.3fZ").unwrap();
        for i in 0..job_num {
            let job_time = NaiveDateTime::parse_from_str(&job_list[i].create_time, "%Y-%m-%dT%H:%M:%S.3fZ").unwrap();
            if job_time.year() > info_time.year() {
                select[i] = false;
            }
            else if job_time.year() == info_time.year() {
                if job_time.month() > info_time.month() {
                    select[i] = false;
                }
                else if job_time.month() == info_time.month() {
                    if job_time.day() > info_time.day() {
                        select[i] = false;
                    }
                    else if job_time.day() == info_time.day() {
                        if job_time.hour() > info_time.hour() {
                            select[i] = false;
                        }
                        else if  job_time.hour() == info_time.hour() {
                            if job_time.minute() > info_time.minute() {
                                select[i] = false;
                            }
                            else if job_time.minute() == info_time.minute() {
                                if job_time.second() > info_time.second() {
                                    select[i] = false;
                                }                               
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(c) = info.state {
        for i in 0..job_num {
            if job_list[i].state.clone() != c {
               select[i] = false;
            }      
        }     
    }

    if let Some(c) = info.result {
        for i in 0..job_num {
            if job_list[i].result.clone() != c {
               select[i] = false;
            }      
        }  
    }

    if let Some(c) = info.user_id {
        for i in 0..job_num {
            if job_list[i].submission.user_id != c {
                select[i] = false;
            }
        }
    }

    if let Some(c) = info.user_name {
        for i in 0..job_num {
            if user_list[job_list[i].submission.user_id as usize].name != c {
                select[i] = false;
            }
        }
    }

    if let Some(c) = info.contest_id {
        for i in 0..job_num {
            if job_list[i].submission.contest_id != c {
                select[i] = false;
            }
        }
    }//contest part

    if let Some(c) = info.problem_id {
        for i in 0..job_num {
            if job_list[i].submission.problem_id != c {
                select[i] = false;
            }
        }
    }

    let mut get_jobs_list: Vec<PostResponseJob> =Vec::new();
    for i in 0..job_num {
        if select[i] == true {
            get_jobs_list.push(job_list[i].clone());
        }
    }
    // select the required jobs by the info
    drop(user_list);
    drop(job_list);

    return HttpResponse::Ok()
    .status(StatusCode::from_u16(200).unwrap())
    .json(get_jobs_list.clone());

}
// fn: get all jobs required from the oj;
// info: the information from the url;
// the response of the http request

#[get("/jobs/{jobid}")]
async fn get_jobsid(jobid: web::Path<String>) -> impl Responder {
    let job_lock = JOB_LIST.lock().unwrap();
    let jobid_num = jobid.parse::<i32>().unwrap();
    let mut exist: bool = false;
    let mut i_num = 0;
    for i in 0..job_lock.len() {
        if job_lock[i].id == jobid_num {
            exist = true;
            i_num = i;
            break;
        }
    }

    if exist == false {
        return HttpResponse::NotFound()
            .status(StatusCode::from_u16(404 as u16).unwrap())
            .json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "Job 123456 not found.".to_string(),
        });
    }

    let responsejob = job_lock[i_num].clone();
    drop(job_lock);

    return HttpResponse::Ok()
    .status(StatusCode::from_u16(200).unwrap())
    .json(responsejob);

}
// fn: get a specific job from the oj;
// jobid: the id of the specific job;
// the response of the http request

#[put("/jobs/{jobid}")]
async fn put_jobsid(jobid: web::Path<String>, config: web::Data<Config>) -> impl Responder {

    let mut job_list = JOB_LIST.lock().unwrap();
    let job_list_num = job_list.len();

    let jobid_num = jobid.parse::<usize>().unwrap();

    if jobid_num >= job_list_num {
        return HttpResponse::NotFound()
        .status(StatusCode::from_u16(404 as u16).unwrap())
        .json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "Job 123456 not found.".to_string(),
        });
    }

    match job_list[jobid_num].state {
        State::Finished => {
           
            let job_now = job_list[jobid_num].clone();
            let body = job_now.submission.clone();

            let mut postresponsejob: PostResponseJob = PostResponseJob { id: 0, create_time: String::new(), updated_time: String::new(),
                submission: body.clone(), state: State::Queueing, result: Result::Waiting, score: 0.0, cases: Vec::new() };
               postresponsejob.cases.push(PostCase { id: 0, result: Result::Waiting, time: 0, memory: 0, info: String::new() });
           //init of postresponsejob
       
           let timec: DateTime<Utc> = Utc::now();
           let mut timec_nano = timec.nanosecond();
           while timec_nano / 1000 > 0 {
               timec_nano = timec_nano / 10;
           }
           let timec_str0 = String::new();
       
           let timec_str = time_reprint(timec, timec_str0, timec_nano);
       
           postresponsejob.create_time = timec_str.clone();
       
           let mut langtest: bool = false;
           let mut lang_num = 0;
           for i in 0..config.languages.len() {
               if config.languages[i].name == body.language {
                   langtest = true;
                   lang_num = i;
                   break;
               }
           }
           //language confirm mod
       
           match langtest {
               true => {
       
                   let mut probtest: bool = false;
                   let mut prob_num = 0;
                   for i in 0..config.problems.len() {
                       if config.problems[i].id == body.problem_id {
                           probtest = true;
                           prob_num = i;
                           break;
                       }
                   }
                   //problem confirm mod
       
                   match probtest {
                       true => {
                           
                            
                           let user_list = USER_LIST.lock().unwrap();
                           if body.user_id > (user_list.len() as i32) - 1 {
                               return HttpResponse::NotFound()
                               .status(StatusCode::from_u16(404 as u16).unwrap())
                               .json(Error {
                                   code: 3,
                                   reason: "ERR_NOT_FOUND".to_string(),
                                   message: String::new(),
                               });
                           }//check the user mod
       
                           let contest_list = CONTEST_LIST.lock().unwrap(); 
                           if body.contest_id > (contest_list.len() as i32) - 1 {
                               return HttpResponse::NotFound()
                               .status(StatusCode::from_u16(404 as u16).unwrap())
                               .json(Error {
                                   code: 3,
                                   reason: "ERR_NOT_FOUND".to_string(),
                                   message: String::new(),
                               });
                           }//check the contset mod 
       
                           else {
       
       
                           }//check the user in the contest mod
                           
                        
       
                           for i in 0..config.problems[prob_num].cases.len() {
                               postresponsejob.cases.push( PostCase { id: i as i32+1, result: Result::Waiting, time: 0, memory: 0, info: String::new() });
                           }//fill the response cases
       
                           let source_code = body.source_code.clone();
       
                           std::fs::create_dir("./TMPDIR");
       
                           let file_before = "TMPDIR/".to_string();
                           let source_file = format!("{}{}",file_before,config.languages[lang_num].file_name.clone());
                           let exe_file = format!("{}{}",file_before,"test.exe".to_string());
                           let out_file = format!("{}{}",file_before,"test.out".to_string());//name of files
       
                           File::create(&source_file);
                           File::create(&exe_file);
                           fs::write(&source_file,source_code);
       
                           //let out_file_f = File::create(&out_file).unwrap();
       
                           let mut command:Vec<String> = config.languages[lang_num].command.clone();
                           for i in &mut command {
                               if i == "%INPUT%" {
                                   *i = source_file.clone();
                               }
                               else if i == "%OUTPUT%" {
                                   *i = exe_file.clone();
                               }
                           }
       
                           postresponsejob.state = State::Running;
                           postresponsejob.result = Result::Running;
                           postresponsejob.cases[0].result = Result::Running;
       
                           let complie_cmd = Command::new(&command[0])
                                                   .args(&command[1..])
                                                   .status()
                                                   .expect("Complication Error");
                                                   //compile mod
       
                    
       
                           if !complie_cmd.success() {
       
                               postresponsejob.state = State::Finished;
                               postresponsejob.result = Result::CompilationError;
                               postresponsejob.cases[0].result = Result::CompilationError;
       
                               let timeu: DateTime<Utc> = Utc::now();
                               let mut timeu_nano = timeu.nanosecond();
                               while timeu_nano / 1000 > 0 {
                                   timeu_nano = timeu_nano / 10;
                               }
                               let timeu_str: String = format!("{}-{}-{}T{}:{}:{}.{}Z", timeu.year(), timeu.month(), timeu.day(), timeu.hour().to_string(),
                               timeu.minute().to_string(), timeu.second().to_string(), timeu_nano.to_string() );
                               postresponsejob.updated_time = timeu_str.clone();
       
                               return HttpResponse::Ok()
                                       .status(StatusCode::from_u16(200 as u16).unwrap())
                                       .json(postresponsejob);         
       
                           }
                           //compile error return 
                           
       
                           postresponsejob.cases[0].result = Result::CompilationSuccess;
       
       
                           let mut score: f64 = 0.0;
       
                           for i in 0..config.problems[prob_num].cases.len() {
       
                               let out_file_f = File::create(&out_file).unwrap();
       
                               let in_file_f = File::open(&config.problems[prob_num].cases[i].input_file).unwrap();
       
                               let test_now = Instant::now();
       
                               let mut test_cmd = Command::new(&exe_file)
                                       //.args([in_file.clone(), out_file.clone()])
                                       .stdin(Stdio::from(in_file_f))
                                       .stdout(Stdio::from(out_file_f))
                                       .spawn()
                                       .unwrap(); //execute & print mode
       
       
                               //runtime error mod
       
                               let test_time = test_now.elapsed().as_micros();
                               postresponsejob.cases[i+1].time = test_time;
       
       
                               let mut time_exceed: bool = false;
       
                               if config.problems[prob_num].cases[i].time_limit != 0 {
                               let usec = Duration::from_micros(config.problems[prob_num].cases[i].time_limit as u64);
                               time_exceed = match test_cmd.wait_timeout(usec).unwrap() {
                                   Some(_) => {
                                       false
                                   }
                                   None => {
                                       true
                                   }
                               };
                               //whether time exceed
       
                               if time_exceed == true {
                                   postresponsejob.cases[i+1].result = Result::TimeLimitExceeded;
                                   match postresponsejob.result {
                                       Result::Running => {
                                           postresponsejob.result = Result::WrongAnswer;
                                       }
                                       _ => {                                          
                                       }
                                   }
                               }
                           }//time exceed process
       
       
                               if time_exceed == false {
                               match config.problems[prob_num].typ {
       
                                   ProblemType::standard => {
                                       let ans_file_buf = File::open(&config.problems[prob_num].cases[i].answer_file).unwrap();
                                       let out_file_buf = File::open(&out_file).unwrap();
                                       let ans_buf = BufReader::new(ans_file_buf);
                                       let out_buf = BufReader::new(out_file_buf);
                                       let mut ans_vec: Vec<String> = Vec::new();
                                       let mut out_vec: Vec<String> = Vec::new();  
       
       
                                       for line in ans_buf.lines() {
                                           ans_vec.push(line.unwrap_or_default());
                                       }
       
                                       for line in out_buf.lines() {
                                           out_vec.push(line.unwrap_or_default());                         
                                       }
       
                                       let mut res: bool = true;
       
                                       for i in 0..min::<usize>(ans_vec.len(),out_vec.len()) {
                                           if ans_vec[i].trim_end() != out_vec[i].trim_end() {
                                               res = false;
                                               break;
                                           }
                                       }
       
                                       if ans_vec.len() > out_vec.len() {
                                           for i in out_vec.len()..ans_vec.len() {
                                               if ans_vec[i].trim_end().len() != 0 {
                                                   res = false;
                                                   break;
                                               }
                                           }
                                       }
       
                                       else if  ans_vec.len() < out_vec.len() {
                                           for i in ans_vec.len()..out_vec.len() {
                                               if out_vec[i].trim_end().len() != 0 {
                                                   res = false;
                                                   break;
                                               }
                                           }
                                       }
       
                                       else {
                                           res = true;
                                       }
       
                                       if res == true {
       
                                           postresponsejob.cases[i+1].result = Result::Accepted;
                                           score += config.problems[prob_num].cases[i].score;
       
                                       }
                                     
                                       else {
                                           postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                           match postresponsejob.result {
                                               Result::Running => {
                                                   postresponsejob.result = Result::WrongAnswer;
                                               }
                                               _ => {                                          
                                               }
                                           }
                                       }
       
                                   }
       
                                   ProblemType::strict => {
                                       
                                       let ans_str: Vec<u8> = std::fs::read(&config.problems[prob_num].cases[i].answer_file).unwrap();
                                       let out_str: Vec<u8> = std::fs::read(&out_file).unwrap();
       
                                       if ans_str == out_str {
       
                                           postresponsejob.cases[i+1].result = Result::Accepted;
                                           score += config.problems[prob_num].cases[i].score;
       
                                       }
       
                                       else {
       
                                           postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                           match postresponsejob.result {
                                               Result::Running => {
                                                   postresponsejob.result = Result::WrongAnswer;
                                               }
                                               _ => {                                          
                                               }
                                           }
       
                                       }
       
                                   }
       
                                   ProblemType::dynamic_ranking => {
                                    if let Some(c) = config.problems[prob_num].misc.clone() {
                                        if let Some(dyn_r) = c.dynamic_ranking_ratio {
    
                                            
                                    let ans_file_buf = File::open(&config.problems[prob_num].cases[i].answer_file).unwrap();
                                    let out_file_buf = File::open(&out_file).unwrap();
                                    let ans_buf = BufReader::new(ans_file_buf);
                                    let out_buf = BufReader::new(out_file_buf);
                                    let mut ans_vec: Vec<String> = Vec::new();
                                    let mut out_vec: Vec<String> = Vec::new();
    
    
                                    for line in ans_buf.lines() {
                                        ans_vec.push(line.unwrap_or_default());
                                    }
    
                                    for line in out_buf.lines() {
                                        out_vec.push(line.unwrap_or_default());                         
                                    }
    
                                    let mut res: bool = true;
    
                                    for i in 0..min::<usize>(ans_vec.len(),out_vec.len()) {
                                        if ans_vec[i].trim_end() != out_vec[i].trim_end() {
                                            res = false;
                                            break;
                                        }
                                    }
    
                                    if ans_vec.len() > out_vec.len() {
                                        for i in out_vec.len()..ans_vec.len() {
                                            if ans_vec[i].trim_end().len() != 0 {
                                                res = false;
                                                break;
                                            }
                                        }
                                    }
    
                                    else if  ans_vec.len() < out_vec.len() {
                                        for i in ans_vec.len()..out_vec.len() {
                                            if out_vec[i].trim_end().len() != 0 {
                                                res = false;
                                                break;
                                            }
                                        }
                                    }
                                    // compare the files line by line
    
                                    if res == true { 
                                        postresponsejob.cases[i+1].result = Result::Accepted;
                                        score += config.problems[prob_num].cases[i].score * (1.0 - dyn_r);
                                    }
                                
                                    else {                                                                    
                                        postresponsejob.cases[i+1].result = Result::WrongAnswer;
                                        match postresponsejob.result {
                                            Result::Running => {
                                                postresponsejob.result = Result::WrongAnswer;
                                            }
                                            _ => {                                          
                                            }
                                        }                            
                                    }
                                        }
                                    }
                                    // giving the score and the state
                                   }
       
                                   ProblemType::spj => {                       

                                    if let Some(c) = config.problems[prob_num].misc.clone() {
                                        if let Some(sp_j) = c.special_judge {
    
                                            let mut command_spj:Vec<String> = sp_j.clone();
    
                                            for j in &mut command_spj {
                                                if j == "%ANSWER%" {
                                                    *j = config.problems[prob_num].cases[i].answer_file.clone();
                                                }
                                                else if j == "%OUTPUT%" {
                                                    *j = "./TMPDIR/test.out".to_string();
                                                }
                                            }
    
                                            let in_time = File::create("./TMPDIR/in_time.in").unwrap();
                                            let out_time = File::create("./TMPDIR/out_time.out").unwrap();
    
                                            let spj_cmd = Command::new(&command_spj[0])
                                                                    .args(&command_spj[1..])
                                                                    .stdin(Stdio::from(in_time))
                                                                    .stdout(Stdio::from(out_time))
                                                                    .status()
                                                                    .unwrap();
                                         
                                            if !spj_cmd.success() {
                                                postresponsejob.cases[i+1].result = Result::SPJError;
    
                                                match postresponsejob.result {
                                                    Result::Running => {
                                                    postresponsejob.result = Result::SPJError;
                                                    }
                                                    _ => {                                          
                                                    }
                                                }
                                            }
    
                                            //compile and run the code
                                            let out_time_open = File::open("./TMPDIR/out_time.out").unwrap();
                                            let out_buf_time = BufReader::new(out_time_open);                                      
                                            let mut out_vec_time: Vec<String> = Vec::new();
          
                                            for line in out_buf_time.lines() {
                                                out_vec_time.push(line.unwrap_or_default());                         
                                            }
                                            
    
                                            if out_vec_time[0] == "Accepted".to_string() {
    
                                                postresponsejob.cases[i+1].result = Result::Accepted;
                                                score += config.problems[prob_num].cases[i].score;
                                                postresponsejob.cases[i+1].info= out_vec_time[1].clone();
                                                
                                            }
    
                                            else if out_vec_time[0] == "Wrong Answer".to_string() {
                                                postresponsejob.cases[i+1].result = Result::WrongAnswer;
    
                                                match postresponsejob.result {
                                                    Result::Running => {
                                                    postresponsejob.result = Result::WrongAnswer;
                                                    }
                                                    _ => {                                          
                                                    }
                                                }
                                                postresponsejob.cases[i+1].info= out_vec_time[1].clone();
                                            }
    
                                            else {
                                                postresponsejob.cases[i+1].result = Result::SPJError;
    
                                                match postresponsejob.result {
                                                    Result::Running => {
                                                    postresponsejob.result = Result::SPJError;
                                                    }
                                                    _ => {                                          
                                                    }
                                                }
                                            }
    
                                            // process the result and push to the struct
                                        }
                                    }
                               
                                }
    
                            }
                        }
                    }
                        

                            std::fs::remove_dir_all("./TMPDIR");
                           
                            let timec: DateTime<Utc> = Utc::now();
                            let mut timec_nano = timec.nanosecond();
                            while timec_nano / 1000 > 0 {
                                timec_nano = timec_nano / 10;
                            }
                            let timec_str0 = String::new();
                        
                            let timec_str = time_reprint(timec, timec_str0, timec_nano);
                        
                            postresponsejob.updated_time = timec_str.clone();
       
                            postresponsejob.state = State::Finished;
                            postresponsejob.score = score;
       
                            match postresponsejob.result {
                                Result::Running =>{
                                    postresponsejob.result = Result::Accepted;
                                }
                                _ => {
        
                                }
                            }
        
                            let timec = job_now.create_time;
                            let timeu = job_now.updated_time;
                            let job_id =job_now.id;
                            job_list[jobid_num] = postresponsejob.clone();
                            job_list[jobid_num].create_time = timec.clone();
                            job_list[jobid_num].updated_time = timeu.clone();
                            job_list[jobid_num].id = job_id;//change area
                            
                            postresponsejob.id = job_id;

       
                            return HttpResponse::Ok()
                            .status(StatusCode::from_u16(200).unwrap())
                            .json(postresponsejob);
                        }
       
                       false => {
                           return HttpResponse::BadRequest()
                           .status(StatusCode::from_u16(404 as u16).unwrap())
                           .json(Error {
                               code: 3,
                               reason: "ERR_NOT_FOUND".to_string(),
                               message: String::new(),
                           });
                       }
                   }
               }
       
               false => {
                   return HttpResponse::BadRequest()
                   .status(StatusCode::from_u16(404 as u16).unwrap())
                   .json(Error {
                       code: 3,
                       reason: "ERR_NOT_FOUND".to_string(),
                       message: String::new(),
                   });
               }
           }


        }//repost the job
        _ => {
            return HttpResponse::BadRequest()
            .status(StatusCode::from_u16(400 as u16).unwrap())
            .json(Error {
                code: 2,
                reason: "ERR_INVALID_STATE".to_string(),
                message: "Job 123456 not finished.".to_string(),
        });
        }
    }
}
// fn: put a job to the oj and retest that;
// jobid: the id of the specific job
// the response of the http request

#[post("/users")]
async fn post_users(user: web::Json<User>) -> impl Responder {

    let mut userlist = USER_LIST.lock().unwrap();

    if let Some(_) = user.id {
        let mut userid_num = 0;
        let mut userid_conf: bool =false;
        for i in 0.. userlist.len() {
            if userlist[i].id == user.id {
                userid_num = i;
                userid_conf = true;
                break;
            }
        }

        if userid_conf == true {

            for i in 0..userlist.len() {
                if userlist[i].name == user.name && i != userid_num {
                    return HttpResponse::BadRequest()
                            .status(StatusCode::from_u16(400 as u16).unwrap())
                            .json(Error {
                                code: 1,
                                reason: "ERR_INVALID_ARGUMENT".to_string(),
                                message: "User name 'root' already exists.".to_string(),
                            });
                }
                // conflict name situation
            }

            userlist[userid_num].name = user.name.clone();
    
            let mut file_data = FILE_DATA.lock().unwrap();
            file_data.users = userlist.clone();
            fs::remove_file("./datafile");
            let mut file = std::fs::File::create("./datafile").unwrap();
            let data_time = file_data.clone();
            serde_json::to_writer(&mut file, &data_time);          
            //realize the persistent storage function
            drop(userlist);
            drop(file_data);

            return HttpResponse::Ok()
            .status(StatusCode::from_u16(200).unwrap())
            .json(user);
            
        }

        else {
            return HttpResponse::NotFound()
            .status(StatusCode::from_u16(404 as u16).unwrap())
            .json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "User 123456 not found.".to_string(),
            });
        }
        //not found the user
    }

    else {
        for i in 0..userlist.len() {
            if userlist[i].name == user.name {
                return HttpResponse::BadRequest()
                .status(StatusCode::from_u16(400).unwrap())
                .json(Error {
                    code: 1,
                    reason: "ERR_INVALID_ARGUMENT".to_string(),
                    message: "User name 'root' already exists".to_string(),
                });
            }
        }

        let userlist_len = userlist.len();
        let user_new = User {id: Some(userlist_len as i32), name: user.name.clone()};
        userlist.push(user_new.clone());

        let mut file_data = FILE_DATA.lock().unwrap();
        file_data.users = userlist.clone();
        fs::remove_file("./datafile");
        let mut file = std::fs::File::create("./datafile").unwrap();
        let data_time = file_data.clone();
        serde_json::to_writer(&mut file, &data_time);
        //realize the function of persistent storage function
        
        drop(userlist);
        drop(file_data);

        return HttpResponse::Ok()
        .status(StatusCode::from_u16(200).unwrap())
        .json(user_new.clone());
        

    }//not exists

}
// fn: post a user to the oj;
// user: the user http json struct;
// the response of the http request

#[get("/users")]
async fn get_users() -> impl Responder {
    let userlist = USER_LIST.lock().unwrap();
    let userlist_copy = userlist.clone();
    drop(userlist);
    return HttpResponse::Ok()
    .status(StatusCode::from_u16(200).unwrap())
    .json(userlist_copy);
}
// fn: get all users from the oj;
// no paras
// the response of the http request

#[post("/contests")]
async fn post_contest(contest: web::Json<Contest>, config: web::Data<Config>) -> impl Responder {

    let mut contest_list = CONTEST_LIST.lock().unwrap();
    if let Some(cid) = contest.id {
        if cid <= contest_list.len() as i32 {

            let user_list = USER_LIST.lock().unwrap();
            let mut wrong: bool = false;
            for i in &contest.problem_ids {
                if i >= &(config.problems.len() as i32) {
                    wrong = true;
                    break;
                }
            }
            for i in &contest.user_ids {
                if i >= &(user_list.len() as i32) {
                    wrong = true;
                    break;
                }
            }

            if wrong == true {
                return HttpResponse::NotFound()
                .status(StatusCode::from_u16(404).unwrap())
                .json(Error {
                    code: 3,
                    reason: "ERR_NOT_FOUND".to_string(),
                    message: "Contest 114514 not found.".to_string(),
                });
            }

            // check the existence of users and problems

            contest_list[cid as usize] = contest.clone();
            
            let mut file_data = FILE_DATA.lock().unwrap();
            file_data.contests = contest_list.clone();

            fs::remove_file("./datafile").unwrap();
            let mut file = std::fs::File::create("./datafile").unwrap();
            let data_time = file_data.clone();
            serde_json::to_writer(&mut file, &data_time).unwrap();          
            // realize the persistent storage function

            drop(file_data);
            drop(contest_list);

            return HttpResponse::Ok()
            .status(StatusCode::from_u16(200).unwrap())
            .json(contest.clone());
        }
        // refresh the contest successfully

        else {
            drop(contest_list);
            return HttpResponse::NotFound()
            .status(StatusCode::from_u16(404).unwrap())
            .json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "Contest 114514 not found.".to_string(),
            });
        }
        // not found the contest

    }

    else {

        let mut contest_new = contest.clone();
        contest_new.id = Some(contest_list.len() as i32);


        let user_list = USER_LIST.lock().unwrap();
        let mut wrong: bool = false;
        for i in &contest_new.problem_ids {
            if i >= &(config.problems.len() as i32) {
                wrong = true;
                break;
            }
        }
        for i in &contest_new.user_ids {
            if i >= &(user_list.len() as i32) {
                wrong = true;
                break;
            }
        }

        if wrong == true {
            return HttpResponse::NotFound()
            .status(StatusCode::from_u16(404).unwrap())
            .json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: "Contest 114514 not found.".to_string(),
            });
        }

        contest_list.push(contest_new.clone());


        let mut file_data = FILE_DATA.lock().unwrap();
        file_data.contests = contest_list.clone();

        fs::remove_file("./datafile").unwrap();
        let mut file = std::fs::File::create("./datafile").unwrap();
        let data_time = file_data.clone();
        serde_json::to_writer(&mut file, &data_time).unwrap();          
        // realize the persistent storage function
        drop(file_data);
        drop(contest_list);

        return HttpResponse::Ok()
        .status(StatusCode::from_u16(200).unwrap())
        .json(contest_new.clone());

    }
}
// fn: post a contest to the oj;
// contest: the contest http json struct, config: the configration file;
// the response of the http request


#[get("/contests")]
async fn get_contest() -> impl Responder {

    let contest_list = CONTEST_LIST.lock().unwrap();
    let mut contestlist_copy: Vec<Contest> = Vec::new();
    let cont_len = contest_list.len();
    for i in 1..cont_len {
        contestlist_copy.push(contest_list[i].clone());
    }
    drop(contest_list);
    return HttpResponse::Ok()
    .status(StatusCode::from_u16(200).unwrap())
    .json(contestlist_copy);

}
// fn: get all contests from the oj;
// no paras
// the response of the http request



#[get("/contests/{contestId}")]
async fn get_contest_id(contestId: web::Path<String>) -> impl Responder {

    let contest_list = CONTEST_LIST.lock().unwrap();
    let contestid = contestId.parse::<i32>().unwrap();

    if contestid < contest_list.len() as i32 {

    let contest_find = contest_list[contestid as usize].clone();
    drop(contest_list);

    return HttpResponse::Ok()
    .status(StatusCode::from_u16(200).unwrap())
    .json(contest_find.clone());

    }


    else {
        drop(contest_list);
        return HttpResponse::NotFound()
        .status(StatusCode::from_u16(404).unwrap())
        .json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "Contest 114514 not found.".to_string(),
        });
    }

}
// fn: get a specific contest from the oj;
// contestId: the id of the specific contest;
// the response of the http request


#[get("/contests/{contestId}/ranklist")]
async fn get_contest_ranklist(contestId: web::Path<String>, web::Query(info):web::Query<ContestRank>, config: web::Data<Config>) -> impl Responder {
    
    let mut userrank_vec: Vec<UserRank> = Vec::new();

    let contestid = contestId.parse::<i32>().unwrap();

    let contest_list = CONTEST_LIST.lock().unwrap();
    let user_list = USER_LIST.lock().unwrap();
    let job_list = JOB_LIST.lock().unwrap();

    let user_len = user_list.len();
    let job_len = job_list.len();

    let mut score_vec: Vec<f64> = Vec::new(); 
    let mut score_pro: Vec<Vec<f64>> = Vec::new();
    let mut score_pro_time: Vec<Vec<NaiveDateTime>> = Vec::new();
    let mut score_pro_timenano: Vec<Vec<i32>>= Vec::new();
    //some inits of the variables

    if contestid == 0 {

        for i in 0..user_len {

            let user = user_list[i].clone();

            score_vec.push(0.0);
            score_pro.push(Vec::new());
            score_pro_time.push(Vec::new());
            score_pro_timenano.push(Vec::new());

            for _ in 0..config.problems.len() {
                score_pro[i].push(0.0);
                score_pro_time[i].push(NaiveDate::from_ymd(1, 1,1).and_hms(1, 1,1));
                score_pro_timenano[i].push(0);
            }


            if let Some(ref c) = info.scoring_rule 
            {

                match c {

                    SRule::latest => {

                        for u in 0..job_len {

                            if job_list[u].submission.user_id == user.id.unwrap() {
                               
                                if let Some(c) = &config.problems[job_list[u].submission.problem_id as usize].misc {
                                    if let Some(k) = c.dynamic_ranking_ratio {
                                        match job_list[u].state {
                                            State::Finished => {

                                                let prob_id_now = job_list[u].submission.problem_id;

                                                let cases_len = config.problems[job_list[u].submission.problem_id as usize].cases.len();
                                                let job_list_len = job_list.len();
                                                let mut score_timecase = 0.0;

                                                score_pro[i][job_list[u].submission.problem_id as usize] = 0.0;

                                                for p in 1..cases_len + 1 {
                                                    let mut time_min: u128 = 0;
                                                    for j in 0..job_list_len {
                                                        if prob_id_now == job_list[j].submission.problem_id &&  job_list[j].submission.user_id == job_list[u].submission.user_id {
                                                            match job_list[j].state {

                                                                State::Finished =>  {
                                                                    if time_min == 0 {
                                                                        time_min = job_list[j].cases[p].time;
                                                                    }
                                                                    if time_min > job_list[j].cases[p].time {
                                                                        time_min = job_list[j].cases[p].time;
                                                                    }
                                                                }
                                                                _ => {
                                                                }
                                                            }
                                                        }
                                                    }  

                                                    score_timecase = 100.0 * k * ((time_min as f64 / job_list[u].cases[p].time as f64)as f64);
                                                    score_pro[i][job_list[u].submission.problem_id as usize] += score_timecase; 
                                                }
                                                
                                                score_pro[i][job_list[u].submission.problem_id as usize] += job_list[u].score ;
                                                score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                                score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap(); 
                                                            
                                            }

                                            _ => {
                                                if score_pro[i][job_list[u].submission.problem_id as usize] >= 100.0 * (1.0 - k) {

                                                }
                                                else {
                                                    score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                                                    score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                                    score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap(); 
                                                }                            
                                            }
                                        }

                                    }
                                    // realizing the dynamic_ratio function


                                    else {
                                        score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                                        score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                        score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap();                             
                                    }
                                }

                                else {
                                score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                                score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap();                             
                                }
                            }
                        }

                        for j in 0..config.problems.len() {
                            score_vec[i] += score_pro[i][j];

                        }
                    }

                    SRule::highest => {

                        for u in 0..job_len {
                       
                            if job_list[u].submission.user_id == user.id.unwrap() {
                               
                                if let Some(c) = &config.problems[job_list[u].submission.problem_id as usize].misc {

                                    if let Some(k) = c.dynamic_ranking_ratio {

                                        match job_list[u].state {
            
                                            State::Finished => {
            
                                                let prob_id_now = job_list[u].submission.problem_id;
            
                                                let cases_len = config.problems[job_list[u].submission.problem_id as usize].cases.len();
                                                let job_list_len = job_list.len();
                                                
            
                                                score_pro[i][job_list[u].submission.problem_id as usize] = 0.0;
            
            
                                                for p in 1..cases_len + 1 {
            
                                                    let mut time_min: u128 = 0;
                                                    let mut score_timecase = 0.0;
            
                                                    for j in 0..job_list_len {
            
                                                        if prob_id_now == job_list[j].submission.problem_id  {
                                                            match job_list[j].state {
            
                                                                State::Finished =>  {
                                                                    if time_min == 0 {
                                                                        time_min = job_list[j].cases[p].time;
                                                                    }
                                                                    if time_min > job_list[j].cases[p].time {
                                                                        time_min = job_list[j].cases[p].time;
                                                                    }
                                                                }
                                                                _ => {
                                                                }
                                                            }
                                                        }
            
                                                    }          
            
                                                    score_timecase = 100.0 * k * ((time_min as f64/ job_list[u].cases[p].time as f64) as f64);
                                                    score_pro[i][job_list[u].submission.problem_id as usize] += score_timecase; 
            
                                                }
            
                                                
                                                score_pro[i][job_list[u].submission.problem_id as usize] += job_list[u].score;
                                                score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                                score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap(); 
                                                    
                                            
                                            }// i is user_id = user.id 
            
                                            _ => {
                                                if score_pro[i][job_list[u].submission.problem_id as usize] >= 100.0 * (1.0 - k) {
            
                                                }
                                                else {              
                                                    if score_pro[i][job_list[u].submission.problem_id as usize] < job_list[u].score {
                 
                                                        score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                                                        score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                                        score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap(); 
                                                    }                            
                                                }
                                            }
                                        }
                                    }

                                    // realizing the dynamic_ratio function


                                        else {

                                            if score_pro[i][job_list[u].submission.problem_id as usize] < job_list[u].score {

                                                score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                                                score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                                score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap();                             
                                        }
                                    }

                                }

                                else {

                                    if score_pro[i][job_list[u].submission.problem_id as usize] < job_list[u].score {

                                        score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                                        score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                        score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap();

                                    }
                                }



                            }

                        }

                        for j in 0..config.problems.len() {
                            score_vec[i] += score_pro[i][j];
                        }
                    }
                }
            }

            else {

                for u in 0..job_len {

                if job_list[u].submission.user_id == user.id.unwrap() {

                    if let Some(c) = &config.problems[job_list[u].submission.problem_id as usize].misc {

                        if let Some(k) = c.dynamic_ranking_ratio {

                            match job_list[u].state  {

                                State::Finished => {

                                    let prob_id_now = job_list[u].submission.problem_id;

                                    let cases_len = config.problems[job_list[u].submission.problem_id as usize].cases.len();
                                    let job_list_len = job_list.len();
                                    

                                    score_pro[i][job_list[u].submission.problem_id as usize] = 0.0;


                                    for p in 1..cases_len + 1 {

                                        let mut time_min: u128 = 0;
                                        let mut score_timecase = 0.0;

                                        for j in 0..job_list_len {

                                            if prob_id_now == job_list[j].submission.problem_id  {
                                                match job_list[j].state {

                                                    State::Finished =>  {
                                                        if time_min == 0 {
                                                            time_min = job_list[j].cases[p].time;
                                                        }
                                                        if time_min > job_list[j].cases[p].time {
                                                            time_min = job_list[j].cases[p].time;
                                                        }
                                                    }
                                                    _ => {
                                                    }
                                                }
                                            }

                                        }          

                                        score_timecase = 100.0 * k * ((time_min as f64/ job_list[u].cases[p].time as f64) as f64);
                                        score_pro[i][job_list[u].submission.problem_id as usize] += score_timecase; 

                                    }

                                    
                                    score_pro[i][job_list[u].submission.problem_id as usize] += job_list[u].score;
                                    score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                    score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap(); 
                                                          
                                }

                                _ => {
                                    if score_pro[i][job_list[u].submission.problem_id as usize] >= 100.0 * (1.0 - k) {

                                    }
                                    else {                               
                                            score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                                            score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                            score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap(); 
                                        }                            
                                    }
                                }
                            }

                             // realizing the dynamic_ratio function

                        else {
                            score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                            score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                            score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap();                             
                        }

                    }

                    else {
                        score_pro[i][job_list[u].submission.problem_id as usize] = job_list[u].score;
                        score_pro_time[i][job_list[u].submission.problem_id as usize] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                        score_pro_timenano[i][job_list[u].submission.problem_id as usize] = job_list[u].create_time[20..23].parse::<i32>().unwrap();                             
                    }               

                }

            }
                for j in 0..config.problems.len() {
                    score_vec[i] += score_pro[i][j];
                }
               
            }
            
        }
        // count the scores and restore them



        if user_len > 1 {

            match info.tie_breaker {

                Some(TBreaker::submission_time) => {

                    let mut score_pro_time_end: Vec<NaiveDateTime> = Vec::new();
                    let mut  score_pro_timenano_end: Vec<i32> = Vec::new();

                    for k in 0..user_len {
                    
                        let mut dat1 = score_pro_time[k][0].clone();
                        let mut dat1nano = score_pro_timenano[k][0].clone();
                        let len0 = score_pro_time[k].len();
                        
                        for h in 1..len0 {
                            if !cmp_date(dat1, score_pro_time[k][h],dat1nano,score_pro_timenano[k][h]) {
                                dat1 = score_pro_time[k][h].clone();
                                dat1nano = score_pro_timenano[k][h];
                            }
                        }

                        if dat1.year() == 1 {
                            score_pro_time_end.push(NaiveDate::from_ymd(2023, 1, 1).and_hms(1, 1, 1));
                            score_pro_timenano_end.push(0);
                        }
                        else {
                            score_pro_time_end.push(dat1.clone());
                            score_pro_timenano_end.push(dat1nano);
                        }

                    }
                    // get the time vec

                    let mut rank0 = 0;
                    let mut max_score = 0.0;
                    
                    loop {

                        let mut set_vec: Vec<i32> = Vec::new();
                        let mut user_rank_time: Vec<UserRank> = Vec::new();         

                        max_score = 0.0;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                            }     
                        }

                        for j in 0..user_len {
                            if max_score == score_vec[j] {
                                set_vec.push(j as i32);
                            }
                        }

                        let set_vec_len = set_vec.len();

                        if set_vec_len == 0 {
                            break;
                        }

                        else if set_vec_len == 1 {
                            rank0 += 1;
                            user_rank_time.push(UserRank { user: user_list[set_vec[0] as usize].clone(), rank: rank0, scores: score_pro[set_vec[0] as usize].clone() });
                        }

                        else {
                        
                        for _ in 0..set_vec_len {

                            let mut ti_num = set_vec[0] as usize;
                            let mut ti = score_pro_time_end[set_vec[0] as usize];
                            let mut tinano = score_pro_timenano_end[set_vec[0] as usize];
                            for b in 0..set_vec_len {
                                if cmp_date(ti, score_pro_time_end[set_vec[b] as usize],tinano,score_pro_timenano_end[set_vec[b] as usize]) {
                                    ti_num = set_vec[b] as usize;
                                    ti = score_pro_time_end[set_vec[b] as usize];
                                    tinano = score_pro_timenano_end[set_vec[b] as usize];
                                }
                            }
                                rank0 += 1;
                                user_rank_time.push(UserRank { user: user_list[ti_num as usize].clone(), rank: rank0, scores: score_pro[ti_num as usize].clone() });
                                score_pro_time_end[ti_num as usize] = NaiveDate::from_ymd(2024, 1, 1).and_hms(1, 1, 1);
                            }

                        }
 
                        for x in user_rank_time {
                            userrank_vec.push(x.clone());
                        }

                        for j in 0..set_vec_len {
                            score_vec[set_vec[j] as usize] = -1.0;
                        }

                    }
                    // rank the users
                }

                Some(TBreaker::submission_count) => {

                    let mut rank0 = 0;
                    let mut max_score = 0.0;
                    
                    loop {

                        let mut set_vec: Vec<i32> = Vec::new();
                        let mut set_num_vec: Vec<i32> = Vec::new();
                        let mut user_rank_time: Vec<UserRank> = Vec::new();         

                        max_score = 0.0;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                            }     
                        }

                        for j in 0..user_len {
                            if max_score == score_vec[j] {
                                set_vec.push(j as i32);
                            }
                        }

                        // rank the counts

                        let set_vec_len = set_vec.len();

                        if set_vec_len == 0 {
                            break;
                        }

                        else if set_vec_len == 1 {
                            rank0 += 1;
                            user_rank_time.push(UserRank { user: user_list[set_vec[0] as usize].clone(), rank: rank0, scores: score_pro[set_vec[0] as usize].clone() });
                        }

                        else {

                            for h in 0..set_vec_len {

                                let num0 = set_vec[h]; 
                                set_num_vec.push(0);

                                for g in 0..job_len {
                                    if job_list[g].submission.user_id == num0 {
                                        set_num_vec[h] += 1;
                                    }
                                }
                            }
         

                        let mut markd: bool = false;
                        let mut mark0: bool = false;
                        let mut last = 0;

                        for _ in 0..set_vec_len {

                            let mut ti_num = 0;
                            let mut ti = set_num_vec[0];

                            if mark0 == true {
                                markd = true;
                            }
                            else {
                                markd = false;
                            }

                            mark0 = false;

                            for b in 0..set_vec_len {

                                if set_num_vec[b] < ti {
                                    ti_num = b;
                                    ti = set_num_vec[b];
                                }

                            }

                            for b in 1..set_vec_len {
                                if set_num_vec[b] == ti && ti != 1000000 && ti_num != b{
                                    mark0 = true;
                                    if set_vec[b] < set_vec[ti_num] {
                                        ti_num = b;
                                        ti = set_num_vec[b];
                                    }
                                }
                            }
                 
                            rank0 += 1;

                            if markd == false {
                                last = rank0;
                                user_rank_time.push(UserRank { user: user_list[set_vec[ti_num] as usize].clone(), rank: rank0, scores: score_pro[set_vec[ti_num] as usize].clone() });
                            }
                            else {      
                                user_rank_time.push(UserRank { user: user_list[set_vec[ti_num] as usize].clone(), rank: last, scores: score_pro[set_vec[ti_num] as usize].clone() });
                            }


                            set_num_vec[ti_num] = 1000000;                           
                            
                        }
                    }
    
                        for x in user_rank_time {
                            userrank_vec.push(x.clone());
                        }


                        for j in 0..set_vec_len {
                            score_vec[set_vec[j] as usize] = -1.0;
                        }

                    }
                    // rank the users
                }

                Some(TBreaker::user_id) => {

                    let mut cid = 0;
                    let mut rank = 0;
                    let mut max_score = 0.0;

                    for i in 0..user_len {

                        max_score = 0.0;
                        cid = 0;
                        rank += 1;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                                cid = j;
                            }
       
                        }
                        

                        if i != 0 && max_score == 0.0 {
                            for j in 0..user_len {
                                if score_vec[j] == 0.0 {
                                    cid = j;
                                    break;
                                }
                            }
                        } 

                        

                        let userrank: UserRank = UserRank { user: user_list[cid].clone(), rank: rank as i32, scores: score_pro[cid].clone() };
                        userrank_vec.push(userrank.clone());
                        score_vec[cid] = -1.0;
                        
                        }
                    }
                
                None => {

                    let mut cid = 0;
                    let mut rank = 0;
                    let mut max_score = 0.0;
                    let mut max_score_last = 0.0;
                    let mut consist = 0;

                    for i in 0..user_len {

                        max_score = 0.0;
                        cid = 0;
                        rank += 1;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                                cid = j;
                            }
                        }

                        
                        if max_score == max_score_last && i != 0 {
                            rank -= 1;
                            consist += 1;
                        }

                        if  i != 0 && max_score == 0.0 {
                            for j in 0..user_len {
                                if score_vec[j] == 0.0 {
                                    cid = j;
                                    break;
                                }
                            }
                        } 
         
                        if consist != 0 {
                            if max_score != max_score_last && i != 0 {
                                rank += consist;
                                consist = 0;
                            }
                        }
                        // check the users and scores and rank in default rules

                        let userrank: UserRank = UserRank { user: user_list[cid].clone(), rank: rank as i32, scores: score_pro[cid].clone()};
                        userrank_vec.push(userrank.clone());

                        score_vec[cid] = -1.0;                     

                        max_score_last = max_score;
                       
                        }
                    }
                }
            }

        else {
            let userrank: UserRank = UserRank { user: user_list[0].clone(), rank: 1, scores: score_pro[0].clone() };
            userrank_vec.push(userrank.clone());
        }
        // count the rank

        return HttpResponse::Ok()
        .status(StatusCode::from_u16(200).unwrap())
        .json(userrank_vec.clone());
        
    }

    // the 0 contest ranklist

    else if contestid > 0 && contestid <= (contest_list.len() as i32) -1 {

        
        let user_len =  contest_list[contestid as usize].user_ids.len();
        let prob_len =  contest_list[contestid as usize].problem_ids.len();
        
        for i in 0..user_len {

            let userid_time = contest_list[contestid as usize].user_ids[i];

            score_vec.push(0.0);
            score_pro.push(Vec::new());
            score_pro_time.push(Vec::new());
            score_pro_timenano.push(Vec::new());

            for _ in 0..prob_len {

                score_pro[i].push(0.0);
                score_pro_time[i].push(NaiveDate::from_ymd(1, 1,1).and_hms(1, 1,1));
                score_pro_timenano[i].push(0);

            }


            if let Some(ref c) = info.scoring_rule 
            {
                match c {

                    SRule::latest => {

                        for u in 0..job_len {

                            if job_list[u].submission.user_id == userid_time && job_list[u].submission.contest_id == contestid {
                               
                                let mut cont = 0;
                                let len_t = contest_list[contestid as usize].problem_ids.len();
                                for i in 0..len_t {
                                    if contest_list[contestid as usize].problem_ids[i] == job_list[u].submission.problem_id {
                                        cont = i;
                                        break;
                                    }
                                }

                                score_pro[i][cont] = job_list[u].score;
                                score_pro_time[i][cont] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                score_pro_timenano[i][cont] = job_list[u].create_time[20..23].parse::<i32>().unwrap();                             
            
                            }
                        }

                        for j in 0..prob_len {
                            score_vec[i] += score_pro[i][j];

                        }
                        
                    }

                    SRule::highest => {

                        for u in 0..job_len {

                            if job_list[u].submission.user_id == userid_time && job_list[u].submission.contest_id == contestid {
                               
                                if score_pro[i][job_list[u].submission.problem_id as usize] < job_list[u].score {

                                    let mut cont = 0;
                                    let len_t = contest_list[contestid as usize].problem_ids.len();
                                    for i in 0..len_t {
                                        if contest_list[contestid as usize].problem_ids[i] == job_list[u].submission.problem_id {
                                            cont = i;
                                            break;
                                        }
                                    }
                                    score_pro[i][cont] = job_list[u].score;
                                    score_pro_time[i][cont] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                                    score_pro_timenano[i][cont] = job_list[u].create_time[20..23].parse::<i32>().unwrap();

                                }
                            }
                        }

                        for j in 0..prob_len {
                            score_vec[i] += score_pro[i][j];
                        }

                    }
                }
            }

            else {

                for u in 0..job_len {

                    if job_list[u].submission.user_id == userid_time && job_list[u].submission.contest_id == contestid {
                                   
                        let mut cont = 0;
                        let len_t = contest_list[contestid as usize].problem_ids.len();
                        for i in 0..len_t {
                            if contest_list[contestid as usize].problem_ids[i] == job_list[u].submission.problem_id {
                                cont = i;
                                break;
                            }
                        }

                        score_pro[i][cont] = job_list[u].score;
                        score_pro_time[i][cont] = NaiveDateTime::parse_from_str(&job_list[u].create_time, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
                        score_pro_timenano[i][cont] = job_list[u].create_time[20..23].parse::<i32>().unwrap();
    
                    }
                }
                

                for j in 0..prob_len {
                    score_vec[i] += score_pro[i][j];
                }

            }
           
        }

        // count and restore the score and restore 
        // this part is just like the 0 contest mod



        if user_len > 1 {

            match info.tie_breaker {

                Some(TBreaker::submission_time) => {

                    let mut score_pro_time_end: Vec<NaiveDateTime> = Vec::new();
                    let mut score_pro_timenano_end: Vec<i32> = Vec::new();

                    for k in 0..user_len {
                    
                        let mut dat1 = score_pro_time[k][0].clone();
                        let mut dat1nano = score_pro_timenano[k][0].clone();
                        let len0 = score_pro_time[k].len();
                        
                        for h in 1..len0 {
                            if !cmp_date(dat1, score_pro_time[k][h],dat1nano,score_pro_timenano[k][h]) {
                                dat1 = score_pro_time[k][h].clone();
                                dat1nano = score_pro_timenano[k][h];
                            }
                        }

                        if dat1.year() == 1 {
                            score_pro_time_end.push(NaiveDate::from_ymd(2023, 1, 1).and_hms(1, 1, 1));
                            score_pro_timenano_end.push(0);
                        }
                        else {
                            score_pro_time_end.push(dat1.clone());
                            score_pro_timenano_end.push(dat1nano);
                        }

                    }
                    // get the vec of time

                    let mut rank0 = 0;
                    let mut max_score = 0.0;
                    
                    loop {

                        let mut set_vec: Vec<i32> = Vec::new();
                        let mut user_rank_time: Vec<UserRank> = Vec::new();         

                        max_score = 0.0;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                            }     
                        }

                        for j in 0..user_len {
                            if max_score == score_vec[j] {
                                set_vec.push(j as i32);
                            }
                        }
                        // rank by score

                        let set_vec_len = set_vec.len();

                        if set_vec_len == 0 {
                            break;
                        }

                        else if set_vec_len == 1 {
                            rank0 += 1;
                            user_rank_time.push(UserRank { user: user_list[contest_list[contestid as usize].user_ids[set_vec[0] as usize] as usize].clone(), rank: rank0, scores: score_pro[set_vec[0] as usize].clone() });
                        }


                        else {
                        
                        for _ in 0..set_vec_len {

                            let mut ti_num = set_vec[0] as usize;
                            let mut ti = score_pro_time_end[set_vec[0] as usize];
                            let mut tinano = score_pro_timenano_end[set_vec[0] as usize];
                            for b in 0..set_vec_len {
                                if cmp_date(ti, score_pro_time_end[set_vec[b] as usize],tinano,score_pro_timenano_end[set_vec[b] as usize]) {
                                    ti_num = set_vec[b] as usize;
                                    ti = score_pro_time_end[set_vec[b] as usize];
                                    tinano = score_pro_timenano_end[set_vec[b] as usize];
                                }
                            }
                                rank0 += 1;
                                user_rank_time.push(UserRank { user: user_list[contest_list[contestid as usize].user_ids[set_vec[ti_num] as usize] as usize].clone(), rank: rank0, scores: score_pro[ti_num as usize].clone() });
                                score_pro_time_end[ti_num as usize] = NaiveDate::from_ymd(2024, 1, 1).and_hms(1, 1, 1);
                            }

                        }
 
                        for x in user_rank_time {
                            userrank_vec.push(x.clone());
                        }

                        for j in 0..set_vec_len {
                            score_vec[set_vec[j] as usize] = -1.0;
                        }

                        // rank by time
                    }
                }

                Some(TBreaker::submission_count) => {

                    let mut rank0 = 0;
                    let mut max_score = 0.0;
                    
                    loop {

                        let mut set_vec: Vec<i32> = Vec::new();
                        let mut set_num_vec: Vec<i32> = Vec::new();
                        let mut user_rank_time: Vec<UserRank> = Vec::new();         

                        max_score = 0.0;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                            }     
                        }

                        for j in 0..user_len {
                            if max_score == score_vec[j] {
                                set_vec.push(j as i32);
                            }
                        }

                        // rank by score

                        let set_vec_len = set_vec.len();

                        if set_vec_len == 0 {
                            break;
                        }

                        else if set_vec_len == 1 {
                            rank0 += 1;
                            user_rank_time.push(UserRank { user: user_list[set_vec[0] as usize].clone(), rank: rank0, scores: score_pro[set_vec[0] as usize].clone() });
                        }

                        else {

                            for h in 0..set_vec_len {

                                let num0 = contest_list[contestid as usize].user_ids[set_vec[h] as usize]; 
                                set_num_vec.push(0);

                                for g in 0..job_len {
                                    if job_list[g].submission.user_id == num0 {
                                        set_num_vec[h] += 1;
                                    }
                                }
                            }
                        
                        let mut markd: bool = false;
                        let mut mark0: bool = false;
                        let mut last = 0;

                        for _ in 0..set_vec_len {

                            let mut ti_num = 0;
                            let mut ti = set_num_vec[0];

                            if mark0 == true {
                                markd = true;
                            }
                            else {
                                markd = false;
                            }

                            mark0 = false;

                            for b in 0..set_vec_len {

                                if set_num_vec[b] < ti {
                                    ti_num = b;
                                    ti = set_num_vec[b];
                                }

                            }

                            for b in 1..set_vec_len {
                                if set_num_vec[b] == ti && ti != 1000000 && ti_num != b {
                                    mark0 = true;
                                    if set_vec[b] < set_vec[ti_num] {
                                        ti_num = b;
                                        ti = set_num_vec[b];
                                    }
                                }
                            }
                            // process the counts vec
                 
                            rank0 += 1;

                            if markd == false {
                                last = rank0;
                                user_rank_time.push(UserRank { user: user_list[contest_list[contestid as usize].user_ids[set_vec[ti_num] as usize] as usize].clone(), rank: rank0, scores: score_pro[set_vec[ti_num] as usize].clone() });
                            }
                            else {      
                                user_rank_time.push(UserRank { user: user_list[contest_list[contestid as usize].user_ids[set_vec[ti_num] as usize] as usize].clone(), rank: last, scores: score_pro[set_vec[ti_num] as usize].clone() });
                            }


                            set_num_vec[ti_num] = 1000000;                           
                            // rank by counts
                        }
                    }
    
                        for x in user_rank_time {
                            userrank_vec.push(x.clone());
                        }


                        for j in 0..set_vec_len {
                            score_vec[set_vec[j] as usize] = -1.0;
                        }

                    }

                }

                Some(TBreaker::user_id) => {

                    let mut cid = 0;
                    let mut rank = 0;
                    let mut max_score = 0.0;

                    for i in 0..user_len {

                        max_score = 0.0;
                        cid = 0;
                        rank += 1;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                                cid = j;
                            }
                        }                     
                        // rank by score
                        if i != 0 && max_score == 0.0 {
                            for j in 0..user_len {
                                if score_vec[j] == 0.0 {
                                    cid = j;
                                    break;
                                }
                            }
                        } 
                        // rank by userid
                        
                        let userrank: UserRank = UserRank { user: user_list[contest_list[contestid as usize].user_ids[cid] as usize].clone(), rank: rank as i32, scores: score_pro[cid].clone() };
                        userrank_vec.push(userrank.clone());
                        score_vec[cid] = -1.0;         
                        }
                    }
                
                None => {

                    let mut cid = 0;
                    let mut rank = 0;
                    let mut max_score = 0.0;
                    let mut max_score_last = 0.0;
                    let mut consist = 0;

                    for i in 0..user_len {

                        max_score = 0.0;
                        cid = 0;
                        rank += 1;

                        for j in 0..user_len {
                            if score_vec[j] > max_score {
                                max_score = score_vec[j];
                                cid = j;
                            }
                        }
                
                        if max_score == max_score_last && i != 0 {
                            rank -= 1;
                            consist += 1;
                        }

                        if  i != 0 && max_score == 0.0 {
                            for j in 0..user_len {
                                if score_vec[j] == 0.0 {
                                    cid = j;
                                    break;
                                }
                            }
                        } 

                                 
                        if consist != 0 {
                            if max_score != max_score_last && i != 0 {
                                rank += consist;
                                consist = 0;
                            }
                        }

                        let userrank: UserRank = UserRank { user: user_list[contest_list[contestid as usize].user_ids[cid] as usize].clone(), rank: rank as i32, scores: score_pro[cid].clone()};
                        userrank_vec.push(userrank.clone());
                        score_vec[cid] = -1.0;                     

                        max_score_last = max_score;
                       
                        }

                        // rank the score as the default rules
                    }
                }
            }

            else {
                let userrank: UserRank = UserRank { user: user_list[contest_list[contestid as usize].user_ids[0] as usize].clone(), rank: 1, scores: score_pro[0].clone() };
                userrank_vec.push(userrank.clone());
            }
            // count the rank

            return HttpResponse::Ok()
            .status(StatusCode::from_u16(200).unwrap())
            .json(userrank_vec.clone());
    }

    // the specific contest rankelist

    else {
        return HttpResponse::NotFound()
        .status(StatusCode::from_u16(404).unwrap())
        .json(Error {
            code: 3,
            reason: "ERR_NOT_FOUND".to_string(),
            message: "Contest 114514 not found.".to_string(),
        });
    }

}
// fn: get the specific ranklist 
// contestId: the id of the contest, info: the information from the url, config: the configration file
// the response of the http request


#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    log::info!(target: "greet_handler", "Greeting {}", name);
    format!("Hello {name}!")
}

// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let mut flush: bool = false;
    let mut config_path: String = String::new();

    let matched = clap::App::new("oj")
                            .arg(clap::Arg::with_name("conf")
                            .short('c')
                            .long("config")
                            .value_name("CONFIG")
                            .help("config command"))
                            .arg(clap::Arg::with_name("flus")
                            .short('f')
                            .long("flush-data")
                            .required(false)
                            .help("flush command"))
                        .get_matches();  
    if let Some(c) = matched.value_of("conf") {
        config_path = c.to_string();
    }   
    if matched.is_present("flus") {
        flush = true;
    }       

    // parsing the commands from the terminal


    let config_file = std::fs::read_to_string(&config_path);
    let configration: Config = serde_json::from_str(&config_file.unwrap())?;


    let file = File::open("./datafile");
    match file {
        Ok(_) => {
            if flush == true {

                fs::remove_file("./datafile");
                File::create("./datafile");

                let mut user_lock = USER_LIST.lock().unwrap();
                user_lock.push(User{ id: Some(0), name:"root".to_string()});
               
            
                let mut contest_lock = CONTEST_LIST.lock().unwrap();
                contest_lock.push( Contest { id: Some(0), name: "root".to_string(), from: "1".to_string(), to: "2".to_string(), problem_ids: Vec::new(), user_ids: Vec::new(), submission_limit: 0 });                
                
                let mut file_data = FILE_DATA.lock().unwrap();
                file_data.contests = contest_lock.clone();
                file_data.users = user_lock.clone();
                fs::remove_file("./datafile").unwrap();
                let mut file = std::fs::File::create("./datafile").unwrap();
                let data_time = file_data.clone();
                serde_json::to_writer(&mut file, &data_time).unwrap();          

                drop(user_lock);
                drop(contest_lock);

            }
            else {

                let mut job_list = JOB_LIST.lock().unwrap();
                let mut user_list = USER_LIST.lock().unwrap();
                let mut contest_list = CONTEST_LIST.lock().unwrap();

                let data_s = std::fs::read_to_string("./datafile").unwrap();
                let data: FileData = serde_json::from_str(&data_s)?;

                let data_job = data.jobs.clone();
                for i in 0..data_job.len() {
                    job_list.push(data_job[i].clone());
                }

                let data_user = data.users.clone();
                for i in 0..data_user.len() {
                    user_list.push(data_user[i].clone());
                }

                let data_contest = data.contests.clone();
                for i in 0..data_contest.len() {
                    contest_list.push(data_contest[i].clone());
                }             

            }
        }
        Err(_) => {

            File::create("./datafile");

            let mut user_lock = USER_LIST.lock().unwrap();
            user_lock.push( User{ id: Some(0), name:"root".to_string()});
 
            let mut contest_lock = CONTEST_LIST.lock().unwrap();
            contest_lock.push( Contest { id: Some(0), name: "root".to_string(), from: "2000-08-27T02:05:29.000Z".to_string(), to: "2050-08-27T02:05:29.000Z".to_string(), problem_ids: Vec::new(), user_ids: Vec::new(), submission_limit: 0 });
            
            let mut file_data = FILE_DATA.lock().unwrap();
            file_data.contests = contest_lock.clone();
            file_data.users = user_lock.clone();
            fs::remove_file("./datafile").unwrap();
            let mut file = std::fs::File::create("./datafile").unwrap();
            let data_time = file_data.clone();
            serde_json::to_writer(&mut file, &data_time);          

            drop(user_lock);
            drop(contest_lock);

        }
    }
    // achieving the persistant storage by files
     

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || { 
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(configration.clone()))
            //.app_data(web::Data::new(userlist.clone()))
            .route("/hello", web::get().to(|| async { "Hello World!" }))
            .service(greet)
            .service(post_jobs)
            .service(get_jobs)
            .service(get_jobsid)
            .service(put_jobsid)
            .service(post_users)
            .service(get_users)
            .service(post_contest)
            .service(get_contest)
            .service(get_contest_id)
            .service(get_contest_ranklist)
            //DO NOT REMOVE: used in automatic testing
            .service(exit)
    })
    .bind(("127.0.0.1", 12345))?
    .run()
    .await
}
