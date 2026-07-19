# Sprint Plan — Minecraft Server in Rust

> 마인크래프트 자바 에디션 서버를 네트워크 프로토콜 밑바닥부터 Rust로 재구현하는 개인 프로젝트.
> **목적 두 가지**: (1) Rust를 실전으로 습득 (2) 게임 서버 도메인(바이너리 프로토콜·상태 머신·틱 루프·동시성) 학습.

## 진행 원칙

- **스프린트 = 데모 하나.** 매 스프린트 끝에 "눈으로 확인되는 결과물"이 하나씩 나오게 스코프를 자른다.
- **스프린트 = Rust 개념 묶음 하나.** 그 스프린트에서 집중적으로 부딪히는 개념을 명시한다.
- 시간으로 못 박지 않는다. 막히면 스프린트 안에서 스코프를 더 쪼갠다.
- 스프린트 끝마다 `handoff.md`에 "배운 Rust 개념 + 다음 액션" 3줄을 남긴다.

## 프로토콜 한눈에

- **프레이밍**: `[VarInt 길이][VarInt 패킷 ID][페이로드]`
- **상태 머신**: Handshake → Status → Login → **Configuration** → Play
  - ⚠️ Configuration은 1.20.2+에서 Login과 Play 사이에 추가된 단계 (레지스트리·리소스팩 협상). 구버전 4단계 기준 아님.
- **오프라인 모드**로 가면 암호화·Mojang 인증을 통째로 건너뛸 수 있다.

## 현재 상태

- [x] 워크스페이스 골격 (`minecraft-protocol` lib + `minecraft-server` bin)
- [x] Cargo 배선: virtual workspace, `resolver = "3"`, 두 크레이트 `edition = "2024"`, server → protocol `path` 참조
- [X] 프로토콜 코드 (0줄, 여기서부터 시작)

---

## Sprint 0 — 배선 확인 + 프로토콜 프리미티브

**데모**: `cargo test` 초록. 서버는 아직 안 뜸.

- [X] `rustc --version` 1.85+ 확인
- [X] `main.rs` / `lib.rs`의 `cargo new` 기본 스텁 제거
- [X] **VarInt** read/write (`minecraft-protocol/src/varint.rs`)
  - 7비트씩 continuation bit, `i32 as u32` 캐스트로 음수 산술 시프트 함정 회피
- [ ] **String** (길이 prefix VarInt + UTF-8), **u16** (포트, big-endian), **i64** (핑 payload)
- [ ] 전부 `impl Read` / `impl Write`에 제네릭하게 (소켓 몰라도 되게)
- [ ] `Cursor<Vec<u8>>` 라운드트립 테스트 — 경계값: `0, 127, 128, 300, i32::MAX, -1, i32::MIN`

**배우는 Rust**: 모듈 시스템(`pub mod varint`), 에러 enum + `From<io::Error>`로 `?` 자동 변환, 제네릭 trait bound(`<W: Write>`). Go의 `io.Writer`와 대응되지만 정적 monomorphization이라는 dispatch 차이.

---

## Sprint 1 — Status 핑 (마일스톤 1)

**데모**: 멀티플레이 목록에 서버가 뜨고 MOTD + 핑(ms)이 찍힘. Join 누르면 멈추는 건 정상.

- [ ] `TcpListener` 블로킹 단일 커넥션 뼈대
- [ ] 상태 머신 `enum State { Handshake, Status }` + `match`
- [ ] Handshake 읽기 → Status Request → Status Response(JSON) → Ping/Pong 에코
- [ ] MOTD·플레이어 수 JSON은 **서버 콘텐츠** → `serde_json` (server 크레이트 의존성)

**배우는 Rust**: `TcpStream` 소유권과 `&mut` 빌림, `match`로 상태 전이(빠진 케이스를 컴파일러가 잡아주는 맛), serde 파생 매크로.

**막히기 쉬운 곳**: 프레이밍에서 길이만큼만 잘라 파싱하는 버퍼 경계 처리.

---

## Sprint 2 — Login (+ Configuration)

**데모**: 클라가 로그인 단계를 통과해 "월드 진입 직전"까지 감(화면은 아직 로딩).

- [ ] 타깃 버전 확정 → 정확한 protocol number를 minecraft.wiki에서 확인
- [ ] 오프라인 모드: `Login Start` → `Login Success`
- [ ] Configuration 단계 패킷 (레지스트리 데이터, known packs 등)

**배우는 Rust**: newtype 패턴(`struct PlayerUuid(u128)`)으로 원시 타입에 의미 입히기, 상태 enum이 커지며 드러나는 refactor seam.

---

## Sprint 3 — Play 진입: 월드에 스폰

**데모**: 클라이언트가 실제로 스폰됨. 평평하거나 텅 빈(void) 세계에 캐릭터가 뜸.

> 첫 번째 큰 벽. 패킷 종류가 확 늘어난다.

- [ ] `Login (Play)` / `Set Default Spawn Position`
- [ ] 최소 chunk 데이터 전송
- [ ] `Synchronize Player Position` (순서 맞춰야 클라가 로딩을 끝냄)
- [ ] 참고 구현(Valence, Pumpkin, FerrumC)을 "막힐 때 훔쳐보는 힌트"로 활용

**배우는 Rust**: 패킷 volume이 늘며 encode/decode dispatch를 enum+match로 갈지 trait object로 갈지의 실전 트레이드오프. 미뤄둔 추상화를 진짜 도입할 타이밍.

---

## Sprint 4 — 틱 루프 + keep-alive + 이동 반영

**데모**: 서버가 20 TPS로 돌고, 클라가 안 튕기고(keep-alive), 내가 움직이면 서버가 위치를 받아 처리.

- [ ] `Instant` / `Duration` 고정 틱 루프 (20 TPS)
- [ ] 주기적 keep-alive 하트비트
- [ ] 이동 패킷 수신·에코
- [ ] "네트워크 I/O와 게임 루프 분리" 구조 고민 시작

**배우는 Rust**: 시간 다루기, 루프 안 상태 소유권, 다음 스프린트 동시성 전환의 워밍업.

---

## Sprint 5 — 동시성: 멀티플레이어 ⭐

**데모**: 둘 이상이 동시 접속해 서로의 접속/이동을 인지.

> 이 프로젝트에서 Rust를 배우는 가장 큰 보상 지점. Go 배경이 대조군으로 빛난다.

**갈림길** (학습 목적이면 위 → 아래 순서 강력 추천):

- **std threads + `mpsc` 채널**: goroutine+channel의 Rust판. `Arc<Mutex<World>>` 공유 상태, `Send`/`Sync`가 컴파일 타임에 데이터 레이스를 막아주는 체감. 학습 밀도 최고.
- **tokio async/await**: 실전 게임 서버에 가깝지만 러닝커브 큼(`Pin`, `.await` 전파, executor).

> 소유권이 스레드 경계를 넘을 때 컴파일러가 왜 화내는지 몸으로 알고 나서 async를 봐야, async가 왜 그렇게 생겼는지 이해된다.

---

## Sprint 6+ — 스트레치 (선택)

취향 영역. 지금 세밀히 안 잡는다.

- 청크 생성·전송
- 블록 배치/파괴
- 인벤토리
- 세계 영속성(디스크 저장)

---

## 참고 자료

- **minecraft.wiki** — 프로토콜 스펙 (버전별 패킷 ID·포맷 확정용)
- **Valence** — Rust 마인크래프트 서버 프레임워크
- **Pumpkin** — Rust 서버 구현
- **FerrumC** — Rust 서버 구현