use sciter::{HELEMENT, Element, Value};
use sciter::dispatch_script_call;
use sciter::make_args;
mod request{
    use std::collections::HashMap;
    pub struct KeyVal{
        pub name:String,
        pub value:String,
        pub extra: HashMap<String,String>,
    }
    impl std::fmt::Display for KeyVal{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
            write!(f, "({}, {},{:?}", self.name, self.value,self.extra);
            Ok(())
        }
    }
    impl KeyVal{
     pub fn new(name:String,value:String)->KeyVal{
         KeyVal{
             name,
             value,
             extra:HashMap::new()
         }
     }
     pub fn with_extra(&mut self,key:String,value:String){
         self.extra.insert(key, value);
     }
    }
    pub struct Request{
        pub url:String,
        pub params:Vec<KeyVal>,
        pub form:Vec<KeyVal>,
        pub headers:Vec<KeyVal>,
    }
    impl std::fmt::Display for Request{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
            write!(f, "({}, {},{},{})", self.url, self.params.len(),self.form.len(),self.headers.len());
            Ok(())
        }
    }
    impl Request{
        pub  fn new(url:String)->Request{
            Request{
                url,
                params:Vec::new(),
                form:Vec::new(),
                headers:Vec::new()
            }
        }
        pub fn add_param(&mut self,name:String,value:String){
            self.params.push(KeyVal::new(name, value))
        }
        pub fn add_header(&mut self,name:String,value:String){
            self.headers.push(KeyVal::new(name, value))
        }
        pub fn add_form(&mut self,name:String,value:String,extras:HashMap<String,String>){
            let mut form=KeyVal::new(name, value);
            for (key,val) in extras.iter(){
                form.with_extra(String::from(key),String::from(val))
            }
            self.form.push(form)
        }
        pub fn get_param_url(&mut self)->Result<String,()>{
            let path: Vec<_> = self.url.split('/').collect();
            let len=path.len();
            let mut param_url=String::from(&self.url);
            if len==1{
                param_url.push_str("/")
            }
            let first_join=path[len-1].contains("?");
            let mut index=0;
            for keyval in &self.params{
                if index==0 && !first_join{
                param_url.push_str("?");
                }else{
                param_url.push_str("&");    
                }
                param_url.push_str(keyval.name.as_str());
                param_url.push_str("=");
                param_url.push_str(keyval.value.as_str());
                index+=1;
            }
            Ok(param_url)
        }
    }
}
macro_rules! request {
    ($method:expr,$req:expr,$self:expr)=> {
        let mut headers = String::from("");
        let mut resp = String::from("");
        let mut error=String::from("");
        let mut is_err=false;
       {
        use curl::easy::{Easy,Form,List};
        let mut easy=Easy::new();
        easy.url($req.get_param_url().unwrap().as_str()).unwrap();
        if $method=="POST"{
        easy.post(true).unwrap();
        }
       let mut list=List::new();
       let mut form=Form::new();
       //form
       for fm in &$req.form{
           let mut part=form.part(fm.name.as_str());
           let tp=fm.extra.get("type").unwrap().as_str();
           if tp=="file"{
               part.file(fm.value.as_str());
           }else{
               part.contents(fm.value.as_bytes());
           }
           part.add().unwrap_or_default();
       }
       easy.httppost(form).unwrap();
       //headers
       for header in &$req.headers{
           let mut header_line=String::new();
           header_line.push_str(header.name.as_str());
           header_line.push_str(": ");
           header_line.push_str(header.value.as_str());
           list.append(header_line.as_str()).unwrap();
       }
       easy.http_headers(list).unwrap();
       let mut transfer=easy.transfer();
       transfer.header_function(|header|{
           headers=std::str::from_utf8(header).unwrap().to_string();
       true
       }).unwrap();
       transfer.write_function(|data|{   
           //small resp
            let resp_result=std::str::from_utf8(data);
            let is_err=resp_result.is_err();
            if is_err{
                resp=String::from("response can't show");
            }else{
                resp=resp_result.unwrap().to_string();
            }
       Ok(data.len())
       }).unwrap();
        
       {
       let result=transfer.perform();
       if result.is_err(){
           is_err=true;
           error=String::from(result.unwrap_err().description());
       }else{
           result.unwrap();
       }
      } 
     }
       if is_err{
           resp=error;
       }
       $self.notify_header(headers);
       $self.notify_resp(resp);
    };
}
macro_rules! foreach {
    ($value:expr,$func:expr) => {
        let mut index=0;
        loop{
            if index>=$value.len(){
                break
            }
            $func($value.get(index));
            index+=1;
        }
    };
}
pub struct EventHandler {
    root: Element,
}
impl EventHandler{
    fn curl(&mut self,method:String,url:String,params:Value,form:Value,headers:Value){
        let mut req=request::Request::new(url);
         foreach!(params,|item:Value|{
             let name=item.get_item("name").as_string().unwrap();
             let value=item.get_item("value").as_string().unwrap();
             req.add_param(name,value);
        });
        foreach!(form,|item:Value|{
            let name=item.get_item("name").as_string().unwrap();
            let value=item.get_item("value").as_string().unwrap();
            let tp=item.get_item("type").as_string().unwrap();
            let mut extra=std::collections::HashMap::new();
            extra.insert(String::from("type"), tp);
            req.add_form(name, value,extra);
       });
        foreach!(headers,|item:Value|{
        let name=item.get_item("name").as_string().unwrap();
        let value=item.get_item("value").as_string().unwrap();
        req.add_header(name,value);
       });
        request!(method,req,self);
    }
    fn notify_header(&mut self,headers:String){
        let value=Value::from(headers);
        self.root.call_function("onHeaderReady",&make_args!(value)).unwrap();
    }
    fn notify_resp(&mut self,resp:String){
        //sciter::CString convert panic
        let value=Value::from(resp.as_bytes());
        self.root.call_function("onRespReady",&make_args!(value)).unwrap();
    }
}

impl sciter::EventHandler for EventHandler {
    fn attached(&mut self, root: HELEMENT) {
		self.root = Element::from(root);
	}
    fn document_complete(&mut self, root: HELEMENT, _target: HELEMENT) {
        self.root = Element::from(root);
    }
    dispatch_script_call! {
         fn curl(String,String,Value,Value,Value);
    }
}
pub fn event_handler() -> Result<EventHandler, ()> {
    let el=Element::create("None").unwrap();
    Ok(EventHandler {root:el})
}

pub struct HostHandler{}

impl sciter::HostHandler for HostHandler{
    fn on_engine_destroyed(&mut self) { 
        println!("bye,see you next time!")
    }
}
pub fn host_handler() -> Result<HostHandler,()>{
    Ok(HostHandler{})
}