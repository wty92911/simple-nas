# Simple Home NAS – High-Level Specification

## 1. Goals
Build a lightweight, self-hosted network-attached storage (NAS) server that runs on consumer hardware, exposes files over HTTPS, and offers a modern web interface for day-to-day file management and media consumption.

## 2. Functional Requirements
- **Authentication**: Local account system; single "admin" role for MVP with future multi-user support.
- **File operations**: Upload, download, rename, move, delete, create folders.
- **Directory browsing**: Tree sidebar, sortable list/grid views, breadcrumb navigation.
- **Storage status**: Display total, used, and free space.
- **Search**: Filename and extension filter with optional recursive search.
- **Sharing**: Generate one-time or time-limited public links.
- **Session persistence**: Remember last opened folder per user.
- **Configurable root path**: Admin selects the physical disk/folder to expose.
- **Large file handling**: Streamed transfers with progress bar and cancel; resume is stretch goal.
- **Background tasks**: Periodic scan to update size metadata and detect external changes.
- **Media service**:
  - Image previews (thumbnails & full-size viewer).
  - In-browser video playback for common codecs (MP4/H.264, WebM/VP9).
  - Optional on-the-fly transcoding or quality selection as stretch goal.

## 3. Non-Functional Requirements
- **Performance**: ≥100 MB/s transfer on Gigabit LAN; <200 ms API latency for metadata calls.
- **Reliability**: No data loss on power failure; atomic writes for uploads and metadata.
- **Security**: HTTPS, Argon2/scrypt password hashing, CSRF protection, input validation, login rate limiting.
- **Portability**: Single binary backend targeting macOS, Linux x86_64, and ARM (e.g., Raspberry Pi).
- **Extensibility**: Modular design to add features like media streaming or cloud sync.
- **Maintainability**: Clear module boundaries, tests, and CI pipeline.
- **Usability**: Responsive UI, drag-and-drop uploads, dark/light themes.

## 4. Constraints
- Runs on ≤2 GB RAM, single CPU core acceptable.
- Local filesystem only; no external object storage.
- No external DB for MVP—use embedded store (SQLite) or file-based metadata.
- Open-source licenses exclusively.

## 5. High-Level Architecture
### Backend (Rust)
- **API layer**: REST/JSON endpoints for auth, file CRUD, metadata, media streams.
- **File manager**: Safe path handling, stream-based I/O.
- **Media module**: Thumbnail generation, video probing/optional transcoding.
- **Auth module**: User store and session tokens (JWT or signed cookies).
- **Share module**: Public link generation & validation.
- **Scheduler**: Background scans and maintenance tasks.
- **Config & logging**.

### Frontend (React)
- **Auth pages**: Login & logout.
- **File explorer**: Folder tree, list/grid, drag-and-drop.
- **Media viewers**: Lightbox for images; HTML5 video player with adaptive quality if available.
- **Transfer queue**: Real-time progress tracking.
- **Settings dashboard**: Storage path, user management (future).
- **Shared-link page**: Minimal file preview & download.

### Data Flow
Browser ⇆ HTTPS ⇆ Rust API ⇆ Local filesystem + metadata store

## 6. Build Phases & Milestones
| Phase | Focus |
|-------|-------|
| 0 | Project scaffolding, repo layout, CI setup |
| 1 | Core MVP: Auth, browse, basic CRUD, storage status, responsive UI |
| 2 | Security & Quality: HTTPS, hashing, CSRF, rate limiting, improved transfers, background scanner |
| 3 | Sharing & Search: Public links, filename search |
| 4 | Media Service: Image thumbnails, video playback |
| 5 | Hardening & Packaging: Cross-compile, Dockerfile, docs, tests |
| 6 | Stretch Enhancements: Multi-user roles, upload resume, video transcoding, cloud backup hooks |

## 7. Acceptance Criteria
- Phase 1 features run without critical bugs on target hardware.
- Sustained transfer ≥100 MB/s on Gigabit LAN.
- Unit/integration test coverage ≥80 % for core modules.
- CI pipeline passes lint, tests, and build for macOS, Linux x86_64, and ARM targets.
- Media phase delivers thumbnail generation and smooth in-browser playback for supported codecs.
