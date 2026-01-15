# Rust 프로젝트 구조 및 아키텍처 설명

이 문서는 `pxm` 프로젝트에 적용된 **Layered Architecture (계층형 아키텍처)**와 Rust 프로젝트의 주요 구성 요소에 대해 설명합니다.

## 1. `src/domain/` (도메인 계층)

- **역할**: 소프트웨어의 **"핵심 재료"**를 정의하는 곳입니다. DB 테이블과 매핑되는 데이터 구조나, 비즈니스 로직에서 사용하는 타입들이 여기에 모입니다.
- **주요 파일**: `approval.rs`
  - `ApprovalRequest`: 결재 요청 1건을 나타내는 구조체입니다. (DB의 `pxm_approval_requests` 테이블과 1:1 대응)
  - `FlowProcess`: 복잡한 결재선 정보를 담는 그릇입니다. DB에는 `JSONB`로 저장되지만, Rust 코드에서는 이 구조체로 다루어 **타입 안정성**을 확보합니다.
- **Rust 특징**: `derive(Serialize, Deserialize)` 같은 매크로를 붙여서, 구조체를 JSON으로 쉽게 변환할 수 있게 합니다.

## 2. `src/repositories/` (리포지토리 계층)

- **역할**: **"데이터 창고지기"**입니다. DB에 직접 접속해서 데이터를 넣고(INSERT), 꺼내고(SELECT), 수정하는 역할을 전담합니다.
- **주요 파일**: `approval_repository.rs`
  - `create`, `find_by_id` 같은 함수들이 있습니다.
  - **이점**: 비즈니스 로직(핸들러) 작성 시, 복잡한 SQL 쿼리를 직접 몰라도 `repo.create()`만 호출하면 되므로 코드가 깔끔해집니다.
- **Rust 특징**: `sqlx::query_as!` 매크로를 사용하여, SQL 쿼리에 오타가 있거나 타입이 안 맞으면 **컴파일 시점에 에러**를 발생시킵니다. (런타임 에러 방지)

## 3. `src/lib.rs` (라이브러리 진입점)

- **역할**: 프로젝트의 **"모듈 수출 관리자"**입니다.
- **필요성**:
  - `src/main.rs`는 "실행 파일(Binary)"을 만드는 곳이고, `src/lib.rs`는 "라이브러리(Library)"를 만드는 곳입니다.
  - **테스트 코드(`tests/`)**는 외부 크레이트(Crate)처럼 취급되므로, `main.rs`의 내용을 직접 가져다 쓸 수 없습니다.
  - 따라서 `lib.rs`를 통해 `domain`과 `repositories`를 외부로 공개(`pub`)해야 테스트 코드와 메인 실행 파일 양쪽에서 사용할 수 있습니다.
- **내용**: `pub mod domain;`, `pub mod repositories;` 등을 선언합니다.

## 4. `mod.rs` (모듈 정의 파일)

- **역할**: 폴더를 하나의 **"패키지(모듈)"**로 묶어주는 리본 같은 존재입니다.
- `src/domain/mod.rs` 파일이 있어야, Rust 컴파일러가 해당 폴더를 모듈로 인식하고 내부의 파일들(`approval.rs` 등)을 관리할 수 있습니다. (Node.js의 `index.js`와 유사)

## 요약

1.  **Domain**: 데이터 구조 정의 (재료)
2.  **Repository**: DB 작업 담당 (창고지기)
3.  **Lib.rs**: 재료와 창고지기를 외부(테스트, 메인)에 공개 (관리 사무소)
4.  **Main.rs**: 서버를 실제로 띄우고 조립 (실행)
