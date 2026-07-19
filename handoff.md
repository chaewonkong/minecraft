# Handoff — Minecraft Server in Rust

> 다음 세션(또는 다른 사람)이 바로 이어갈 수 있게 프로젝트 현재 상태를 정리한 문서.
> 큰 로드맵은 루트의 `sprint-plan.md` 참고. 이 문서는 "지금 어디까지 왔고, 왜 이렇게 짰고, 다음에 뭘 하나"에 집중.

## 프로젝트 한 줄

마인크래프트 자바 에디션 서버를 네트워크 프로토콜 밑바닥부터 Rust로 재구현하는 개인 프로젝트.
**목적 두 가지**: (1) Rust 실전 습득 (2) 게임 서버 도메인(바이너리 프로토콜·상태 머신·틱 루프·동시성) 학습.
배경: Go 백엔드 엔지니어. Rust는 개념·문법을 어렴풋이 아는 상태에서 만들며 배우는 중.

## 현재 상태 요약

- **Sprint 0 완료** ✅ — 프로토콜 프리미티브 (VarInt, u16, i64, String) + 라운드트립 테스트
- **Sprint 1 완료 = 마일스톤 1 달성** ✅ — Status 핑. 멀티플레이 목록에 MOTD·0/20·초록핑 표시
- **다음: Sprint 2** — Login (오프라인 모드) + Configuration 단계

## 프로젝트 구조

```
minecraft/                    ← git repo 루트 (virtual workspace)
├── Cargo.toml                ← resolver = "3", members = [server, protocol]
├── sprint-plan.md            ← 전체 로드맵 (Sprint 0~6+)
├── handoff.md                ← 이 문서
├── minecraft-protocol/       ← lib 크레이트 (edition 2024)
│   └── src/
│       ├── lib.rs            ← pub mod varint / numbers / string
│       ├── varint.rs         ← VarInt read/write + VarIntError + Display/Error impl
│       ├── numbers.rs        ← u16 / i64 read/write (big-endian 고정 길이)
│       └── string.rs         ← String read/write (VarInt 길이 prefix + UTF-8)
└── minecraft-server/         ← bin 크레이트 (edition 2024)
    └── src/
        └── main.rs           ← TcpListener + handle_connection + handle_status
```

## 핵심 설계 결정

- **크레이트 분리**: `minecraft-protocol`(순수 프로토콜 타입/인코딩)과 `minecraft-server`(I/O·게임 로직)로 분리. 프로토콜 크레이트는 `TcpStream`을 전혀 모름.
- **프리미티브는 `impl Read`/`impl Write`에 제네릭**(`<R: Read>`, `<W: Write>`). 덕분에 소켓 없이 `Cursor<Vec<u8>>`로 테스트 가능하고, write 시 payload를 `Vec<u8>` 버퍼에 조립 후 프레이밍 가능. (Sprint 1에서 이 설계가 실전 이득으로 회수됨.)
- **에러 처리**: 프로토콜 계층은 `VarIntError` enum(`Io`, `TooLong`) + `From<io::Error>` + `Display` + `Error` impl. 서버 계층은 `Box<dyn Error>`로 여러 에러 타입을 통합, `?`로 전파.
- **커넥션 격리**: `if let Err(e) = handle_connection(stream)`로 커넥션별 에러를 로그만 찍고 서버는 계속 실행. 한 클라의 깨진 입력이 서버 전체를 못 죽임. (실제로 레거시 핑 패킷이 왔을 때 검증됨.)

## 프로토콜 참고 (확정된 사실)

- **프레이밍**: `[VarInt 길이][VarInt 패킷ID][payload]`. VarInt/VarLong 제외 모든 데이터는 big-endian.
- **상태 머신**: Handshake → Status → Login → **Configuration** → Play (Configuration은 1.20.2+ 추가; 구버전 4단계 아님).
- **Handshake 필드**: VarInt 프로토콜버전, String 주소, u16 포트, VarInt next_state (1=Status, 2=Login, 3=Transfer).
- **Status 4메시지**: C→Status Request(id=0x00) / S→Status Response(id=0x00, String=JSON) / C→Ping(id=0x01, i64) / S→Pong(id=0x01, 같은 i64 에코).
- **테스트 클라 = Minecraft 26.2, protocol number = 776.** Status Response JSON의 `version.protocol`에 776을 넣어야 "호환됨(초록막대)"으로 뜸. protocol 일치 시 클라는 버전 이름을 굳이 표시 안 함(정상).
- 핑 ms는 목록에 숫자로 안 뜨고 신호막대로만 표시. 숫자는 막대에 호버 시 툴팁.

## 지금까지 배운 Rust (개념 인덱스)

- **소유권/빌림**: move vs `&`(읽기 빌림) vs `&mut`(쓰기 빌림). 함수 파라미터에서 `mut stream: T`(소유권+가변) vs `&mut T`(빌림+가변) 구분. `&mut`는 동시에 하나만, `&`와 `&mut` 공존 불가 (NLL = 마지막 사용 지점까지).
- **Copy vs move 타입**: `[u8; 2]`·정수는 Copy(값으로 넘겨도 원본 살아있음), `String`·`Vec`·소켓은 move.
- **enum = 합타입(sum type)** + `match` 망라 검사. Go의 kind 필드/인터페이스 흉내와 대비.
- **에러**: `Result`/`?` 전파, `From`으로 자동 변환, `Display`+`Error` trait(supertrait: Debug+Display), `Box<dyn Error>`(trait object, 동적 dispatch).
- **제네릭 trait bound** `<W: Write>` = 정적 monomorphization (Go 인터페이스의 동적 dispatch와 대비). 컴파일 시간 대가의 원인이기도.
- **모듈 시스템**: 파일 트리 ≠ 모듈 트리, `pub mod`, `crate::`, 크레이트 넘어 import(하이픈→밑줄: `minecraft_protocol`).
- **비트 연산**: `& 0x7F`(남기기), `| 0x80`(켜기), `& 0x80`(검사), 논리 vs 산술 시프트(`as u32` 캐스트), 1byte=8bit.
- **엔디언**: `to_be_bytes`/`from_be_bytes`, network byte order = big-endian.
- **타입/문법**: `usize`/`u8`/`i32`/`i64` 비트폭, `vec![0u8; n]`(동적 버퍼, Go `make`), `[0u8; n]`(고정 배열), raw string `r#"..."#`, `if let` vs `match`, shadowing, `Self`, `::`(연관 함수) vs `.`(메서드).
- **cargo**: `-p <crate>`로 워크스페이스 대상 지정, `cargo check`(빠른 타입/borrow 검사), 증분 컴파일.

## 남긴 빚 (technical debt)

Sprint 2~3에서 정리하면 좋은 것들. 지금은 M1 동작에 지장 없음.

- [ ] **JSON 하드코딩** → `serde_json`으로 리팩터 (players.online 실시간 값 등 필요해지면).
- [ ] **`read_string`의 UTF-8 실패를 `VarIntError::TooLong`으로 임시 매핑** → `InvalidUtf8` 변형 추가로 의미 정정.
- [ ] **`packet_id` 검증 안 함** — Handshake/Status Request 등에서 id가 기대값인지 확인 안 하고 읽어서 버림.
- [ ] **`_length` 미사용** — 지금은 스트림에서 필드 직접 읽음. 더 방어적으로는 length만큼 버퍼에 read_exact 후 Cursor로 파싱.
- [ ] **레거시 핑 미처리** — `next_state`가 1/2/3 아닌 쓰레기값으로 올 때(로컬 탐색 패킷) "알 수 없음" 로그만. 감지해서 조용히 닫기.
- [ ] **Ping 안 오는 경우** — Status 후 클라가 Ping 없이 끊으면 `read_varint`가 EOF 에러(현재는 격리돼서 안전하지만 로그가 에러로 남음).

## 다음: Sprint 2 (Login + Configuration)

- **목표**: Handshake `next_state=2` 갈래를 채워, 클라가 로그인 단계를 통과해 "월드 진입 직전"까지.
- **오프라인 모드**로 암호화·Mojang 인증 건너뜀: `Login Start` → `Login Success`.
- **Configuration 단계**(26.2에 존재): 레지스트리 데이터·known packs 협상. 이걸 해줘야 클라가 Play로 넘어감.
- **착수 전 확인**: minecraft.wiki에서 protocol 776 기준 Login/Configuration 패킷 ID·포맷·순서 확정.
- Sprint 1보다 패킷 수·순서 복잡도 상승. i64 외에 UUID(u128) 프리미티브가 필요해질 수 있음.

## 참고 자료

- **minecraft.wiki** — 프로토콜 스펙 (버전별 패킷 ID·포맷). Server List Ping / Packets / FAQ 페이지.
- **Valence**, **Pumpkin**, **FerrumC** — Rust 마인크래프트 서버 구현. "정답지"가 아니라 막힐 때 힌트로.

## 빌드/실행

```bash
cargo test -p minecraft-protocol      # 프리미티브 라운드트립 테스트
cargo run  -p minecraft-server        # localhost:25565 서버 실행
# 마인크래프트 → 멀티플레이 → localhost 추가 → Refresh
```