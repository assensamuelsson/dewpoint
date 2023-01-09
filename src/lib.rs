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

    let dewpoint = calc_dewpoint(trh);

    write_success(&mut stream, dewpoint);
}

fn calc_dewpoint(trh: TRH) -> f64 {
    let b = 17.67;
    let c = 243.5;

    let gamma = (100.0/trh.rh).ln() + (b * trh.t) / (c + trh.t);

    c * gamma / (b - gamma)
}

fn parse_request(request: &String) -> Result<TRH, String> {
    if !request.starts_with("GET /dewpoint/") {
        return Err(String::from("Only GET to /dewpoint is allowed!"));
    }

    let path = request.split(" ").nth(1).unwrap();
    let parts: Vec<&str> = path.split("/").collect();

    let t = match parts.get(2) {
        Some(t) => t,
        None => return Err(String::from("t or rh is missing! Request must be /dewpoint/{t}/{rh}"))
    };
    let rh = match parts.get(3) {
        Some(rh) => rh,
        None => return Err(String::from("t or rh is missing! Request must be /dewpoint/{t}/{rh}"))
    };

    let t = match t.parse::<f64>() {
        Ok(t) => t,
        Err(_)=> return Err(String::from(format!("Cannot convert t to a float! Got '{}'!", parts[2]))),
    };
    let rh= match rh.parse::<f64>() {
        Ok(rh) => rh,
        Err(_)=> return Err(String::from(format!("Cannot convert rh to a float! Got '{}'!", parts[3]))),
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

fn write_success(stream: &mut TcpStream, dewpoint: f64) {
    let status_line = "HTTP/1.1 200 OK";
    let body = format!("{{\"dewpoint\":\"{dewpoint}\"}}");
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
        assert_eq!(parse_request(&String::from("GET /dewpoint/15.7/87.1 HTTP/1.1")).unwrap(), TRH { t: 15.7, rh: 87.1 });
        assert_eq!(parse_request(&String::from("GET /dewpoint/-3.2/97.3 HTTP/1.1")).unwrap(), TRH { t: -3.2, rh: 97.3 });
        assert_eq!(parse_request(&String::from("GET /dewpoint/0/0 HTTP/1.1")).unwrap(), TRH { t: 0f64, rh: 0f64 });
    }

    #[test]
    fn it_should_err_if_request_is_anything_but_get_to_depoint() {
        assert_eq!(parse_request(&String::from("POST /dewpoint/15.7/87.1 HTTP/1.1")).err().unwrap(), "Only GET to /dewpoint is allowed!");
        assert_eq!(parse_request(&String::from("GET /calcdewpoint/15.7/87.1 HTTP/1.1")).err().unwrap(), "Only GET to /dewpoint is allowed!");
    }

    #[test]
    fn it_should_err_if_t_or_rh_cannot_be_converted_to_floats() {
        assert_eq!(parse_request(&String::from("GET /dewpoint/foo/87.1 HTTP/1.1")).err().unwrap(), "Cannot convert t to a float! Got 'foo'!");
        assert_eq!(parse_request(&String::from("GET /dewpoint/15.7/bar HTTP/1.1")).err().unwrap(), "Cannot convert rh to a float! Got 'bar'!");
    }

    #[test]
    fn it_should_err_if_t_or_rh_is_outside_valid_boundaries() {
        assert_eq!(parse_request(&String::from("GET /dewpoint/1234/87.1 HTTP/1.1")).err().unwrap(), "t is too high! Got '1234'! Max allowed t is 80!");
        assert_eq!(parse_request(&String::from("GET /dewpoint/-1234/87.1 HTTP/1.1")).err().unwrap(), "t is too low! Got '-1234'! Min allowed t is -40!");
        assert_eq!(parse_request(&String::from("GET /dewpoint/15.7/100.1 HTTP/1.1")).err().unwrap(), "rh is too high! Got '100.1'! Max allowed rh is 100!");
        assert_eq!(parse_request(&String::from("GET /dewpoint/15.7/-0.1 HTTP/1.1")).err().unwrap(), "rh is too low! Got '-0.1'! Min allowed rh is 0!");
    }

    #[test]
    fn it_should_err_if_rh_or_t_is_missing() {
        assert_eq!(parse_request(&String::from("GET /dewpoint/")).err().unwrap(), "t or rh is missing! Request must be /dewpoint/{t}/{rh}");
    }
}