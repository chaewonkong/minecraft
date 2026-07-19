use std::error::Error;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};

use minecraft_protocol::numbers::{read_i64, read_u16, write_i64};
use minecraft_protocol::string::{read_string, write_string};
use minecraft_protocol::varint::{read_varint, write_varint};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565")?;
    println!("Listening on 127.0.0.1:25565");

    for stream in listener.incoming() {
        let stream = stream?;
        println!("New connection from {}", stream.peer_addr()?);

        // stream의 소유권이 handle_connection으로 이동. 아래와 동일한 구현의 축약버전임.
        //         match handle_connection(stream) {
        //     Ok(()) => {}                                  // 성공 → 아무것도 안 함
        //     Err(e) => eprintln!("  connection error: {e}"),
        // }
        if let Err(e) = handle_connection(stream) {
            // if let Err(e): result가 Err 패턴에 맞으면 e를 꺼내 실행하는 축약 문법
            eprintln!("  connection error: {e}"); // 커넥션 하나가 죽어도 서버는 계속
        }
    }

    Ok(())
}

// mut stream: TcpStream: 소유권을 받음. mut이 붙었으므로 가변
fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // 읽기순서: 프레임길이 → 패킷ID → 프로토콜버전 → 주소(String) → 포트(u16) → next_state

    // 프레임 길이
    let _length = read_varint(&mut stream)?;
    let packet_id = read_varint(&mut stream)?; // handshake면 0

    // Handshake 필드
    let protocol_version = read_varint(&mut stream)?;
    let server_address = read_string(&mut stream)?;
    let server_port = read_u16(&mut stream)?;
    let next_state = read_varint(&mut stream)?;

    println!(
        "  Handshake: id={packet_id}, proto={protocol_version}, \
         addr={server_address}, port={server_port}, next_state={next_state}"
    );

    match next_state {
        1 => handle_status(&mut stream)?,
        2 => println!("  → Login 시도 (실제 접속)"),
        other => println!("  → 알 수 없는 next_state: {other}"),
    }

    Ok(())
}

fn handle_status(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    // 프레임 길이
    let _length = read_varint(stream)?;
    let _packet_id = read_varint(stream)?;

    // 2) Status Response 보낼 JSON: protocol 776은 사전정의 된 것.
    let json = r#"{"version":{"name":"26.2","protocol":776},"players":{"max":20,"online":0},"description":{"text":"Rust로 만든 서버!"}}"#;

    // payload 조립: [packet id = 0x00][String(json)]
    let mut payload = Vec::new();
    write_varint(&mut payload, 0x00)?;
    write_string(&mut payload, json)?;

    write_varint(stream, payload.len() as i32)?;
    stream.write_all(&payload)?;

    println!("  → Status Response 전송 ({} bytes)", payload.len());

    let _length = read_varint(stream)?;
    let _packet_id = read_varint(stream)?; // 1
    let ping_payload = read_i64(stream)?; // 클라가 보낸 타임스탬프

    // 4) Pong Response 쓰기 — 같은 i64를 그대로 에코
    let mut pong = Vec::new();
    write_varint(&mut pong, 0x01)?; // Pong packet ID = 1
    write_i64(&mut pong, ping_payload)?; // 받은 값 그대로

    write_varint(stream, pong.len() as i32)?;
    stream.write_all(&pong)?;

    println!("  → Pong Response 전송 (payload={ping_payload})");
    Ok(())
}
