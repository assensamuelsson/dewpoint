use std::{io::{prelude::*, BufReader}, net::TcpStream};

pub fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .take(1)
        .map(|result| result.unwrap())
        .collect();

    let trh = match parse_request(&http_request[0]) {
        Ok(trh) => trh,
        Err(message) => return write_err(&mut stream, 400, message.as_str())
    };

    let dewpoint = calc_dewpoint(&trh);
    let mould_index = calc_mould_index(&trh);

    write_success(&mut stream, dewpoint, mould_index);
}

fn calc_dewpoint(trh: &TRH) -> f64 {
    let b = 17.67;
    let c = 243.5;

    let gamma = (100.0/trh.rh).ln() + (b * trh.t) / (c + trh.t);

    c * gamma / (b - gamma)
}

fn calc_mould_index(trh: &TRH) -> i32 {
    let t = trh.t.round() as i32;
    let rh = trh.rh.round() as i32;
    static TABLE: [[i32; 4]; 51] = [
        [0,0,0,0],     // 0°
        [0,97,98,100], // 1°
        [0,95,97,100], // 2°
        [0,93,95,100], // 3°
        [0,91,93,98],  // 4°
        [0,88,92,97],  // 5°
        [0,87,91,96],  // 6°  
        [0,86,91,95],  // 7°  
        [0,84,90,95],  // 8°  
        [0,83,89,94],  // 9°  
        [0,82,88,93],  // 10°  
        [0,81,88,93],  // 11°  
        [0,81,88,92],  // 12°  
        [0,80,87,92],  // 13°  
        [0,79,87,92],  // 14°  
        [0,79,87,91],  // 15°  
        [0,79,86,91],  // 16°  
        [0,79,86,91],  // 17°  
        [0,79,86,90],  // 18°  
        [0,79,85,90],  // 19°  
        [0,79,85,90],  // 20°  
        [0,79,85,90],  // 21°  
        [0,79,85,89],  // 22°  
        [0,79,84,89],  // 23°  
        [0,79,84,89],  // 24°
        [0,79,84,89],  // 25°  
        [0,79,84,89],  // 26°  
        [0,79,83,88],  // 27°  
        [0,79,83,88],  // 28°  
        [0,79,83,88],  // 29°  
        [0,79,83,88],  // 30°  
        [0,79,83,88],  // 31°  
        [0,79,83,88],  // 32°  
        [0,79,82,88],  // 33°  
        [0,79,82,87],  // 34°  
        [0,79,82,87],  // 35°  
        [0,79,82,87],  // 36°  
        [0,79,82,87],  // 37°  
        [0,79,82,87],  // 38°  
        [0,79,82,87],  // 39°  
        [0,79,82,87],  // 40°  
        [0,79,81,87],  // 41°  
        [0,79,81,87],  // 42°  
        [0,79,81,87],  // 43°  
        [0,79,81,87],  // 44°  
        [0,79,81,86],  // 45°  
        [0,79,81,86],  // 46°  
        [0,79,81,86],  // 47°  
        [0,79,80,86],  // 48°  
        [0,79,80,86],  // 49°  
        [0,79,80,86]   // 50°
    ];

    let mut mould_index: i32= 0;
    if t <= 0 || t > 50 {
        mould_index = 0;
    } else {
        let mut mould_index_set = false;
        for i in 1..4 {
            if rh <= TABLE[t as usize][i] {
                mould_index = i as i32 - 1;
                mould_index_set = true;
                break;
            }
        }

        if !mould_index_set {
            mould_index = 3;
        }
    }
    return mould_index;
}

fn parse_request(request: &String) -> Result<TRH, String> {
    let path = request.split(" ").nth(1).unwrap();
    let parts: Vec<&str> = path.split("/").collect();

    let t = match parts.get(1) {
        Some(t) => t,
        None => return Err(String::from("t or rh is missing! Request must be /{t}/{rh}"))
    };
    let rh = match parts.get(2) {
        Some(rh) => rh,
        None => return Err(String::from("t or rh is missing! Request must be /{t}/{rh}"))
    };

    let t = match t.parse::<f64>() {
        Ok(t) => t,
        Err(_)=> return Err(String::from(format!("Cannot convert t to a float! Got '{}'!", t))),
    };
    let rh= match rh.parse::<f64>() {
        Ok(rh) => rh,
        Err(_)=> return Err(String::from(format!("Cannot convert rh to a float! Got '{}'!", rh))),
    };

    if t > 80.0 {
        return Err(String::from(format!("t is too high! Got '{t}'! Max allowed t is 80!")));
    } else if t < -40.0 {
        return Err(String::from(format!("t is too low! Got '{t}'! Min allowed t is -40!")));
    } else if rh > 100.0 {
        return Err(String::from(format!("rh is too high! Got '{rh}'! Max allowed rh is 100!")));
    } else if rh < 0.0 {
        return Err(String::from(format!("rh is too low! Got '{rh}'! Min allowed rh is 0!")));
    }

    Ok(TRH{ t, rh })
}

fn write_err(stream: &mut TcpStream, code: i32, message: &str) {
    let code_description = match code {
        400 => "Bad Request",
        _ => "Internal Server Error",
    };

    let status_line = format!("HTTP/1.1 {code} {code_description}");
    let body = format!("{{\"message\":\"{message}\"}}");
    let length = body.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{body}");
    
    stream.write_all(response.as_bytes()).unwrap();
}

fn write_success(stream: &mut TcpStream, dewpoint: f64, mould_index: i32) {
    let status_line = "HTTP/1.1 200 OK";
    let body = format!("{{\"dewpoint\":{dewpoint}, \"mould_index\":{mould_index}}}");
    let length = body.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{body}");
    
    stream.write_all(response.as_bytes()).unwrap();
}

#[derive(PartialEq, Debug)]
struct TRH {
    t: f64,
    rh: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_parse_request_correctly() {
        assert_eq!(parse_request(&String::from("GET /15.7/87.1 HTTP/1.1")).unwrap(), TRH { t: 15.7, rh: 87.1 });
        assert_eq!(parse_request(&String::from("GET /-3.2/97.3 HTTP/1.1")).unwrap(), TRH { t: -3.2, rh: 97.3 });
        assert_eq!(parse_request(&String::from("GET /0/0 HTTP/1.1")).unwrap(), TRH { t: 0f64, rh: 0f64 });
    }

    #[test]
    fn it_should_err_if_t_or_rh_cannot_be_converted_to_floats() {
        assert_eq!(parse_request(&String::from("GET /foo/87.1 HTTP/1.1")).err().unwrap(), "Cannot convert t to a float! Got 'foo'!");
        assert_eq!(parse_request(&String::from("GET /15.7/bar HTTP/1.1")).err().unwrap(), "Cannot convert rh to a float! Got 'bar'!");
    }

    #[test]
    fn it_should_err_if_t_or_rh_is_outside_valid_boundaries() {
        assert_eq!(parse_request(&String::from("GET /1234/87.1 HTTP/1.1")).err().unwrap(), "t is too high! Got '1234'! Max allowed t is 80!");
        assert_eq!(parse_request(&String::from("GET /-1234/87.1 HTTP/1.1")).err().unwrap(), "t is too low! Got '-1234'! Min allowed t is -40!");
        assert_eq!(parse_request(&String::from("GET /15.7/100.1 HTTP/1.1")).err().unwrap(), "rh is too high! Got '100.1'! Max allowed rh is 100!");
        assert_eq!(parse_request(&String::from("GET /15.7/-0.1 HTTP/1.1")).err().unwrap(), "rh is too low! Got '-0.1'! Min allowed rh is 0!");
    }

    #[test]
    fn it_should_err_if_rh_or_t_is_missing() {
        assert_eq!(parse_request(&String::from("GET /")).err().unwrap(), "t or rh is missing! Request must be /{t}/{rh}");
    }
}