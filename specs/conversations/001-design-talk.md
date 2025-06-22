# 1
User:
```
I want to design a simple nas for my home server with backend using rust and frontend using react.
I will use vibe coding with cursor and memory bank to build the project. So at first, I need you to identify core requirements and make a spec.
In this phase, you donot need to be too specific, focus on the requirements and a plan.
More specific todos and tech selection are the next.

```

AI:
```

High-Level Specification & Build Plan  
Project: Simple Home-Server NAS  
Stack: Rust backend, React frontend (exact frameworks/tools chosen in later phase)

1. Functional Requirements  
   • User authentication: local account system with password login; single “admin” role sufficient for MVP, with future multi-user support.  
   • File operations: upload, download, rename, move, delete, create folders.  
   • Directory browsing: tree-view of folders, sortable list/grid views, breadcrumb navigation.  
   • Storage status: total/used/free space indicators.  
   • Search: filename and extension filter, optional recursive search.  
   • Basic sharing: generate one-time or time-limited public link.  
   • Session persistence: remember last opened folder per user.  
   • Configurable root path: admin selects physical disk/folder to expose.  
   • Graceful handling of large files: progress bar, cancel, resume (MVP requires progress & cancel; resume is stretch).  
   • Background tasks: periodic scan to update size metadata and detect external changes.

2. Non-Functional Requirements  
   • Performance: sustain ≥100 MB/s transfer on Gigabit LAN (limited by disk) with <200 ms API latency for metadata actions.  
   • Reliability: no data loss on unexpected shutdown; atomic writes for uploads and metadata updates.  
   • Security: HTTPS, password hashing (Argon2 or scrypt), CSRF protection, input validation, rate limiting on login.  
   • Portability: single binary backend; runs on macOS, Linux x86_64, and ARM (e.g., Raspberry Pi).  
   • Extensibility: plugin-style architecture for future additions (e.g., media streaming, cloud sync).  
   • Maintainability: clear module boundaries, unit/integration tests, CI pipeline.  
   • Usability: responsive UI, drag-and-drop uploads, dark/light themes.

3. Constraints  
   • Runs on consumer hardware (≤2 GB RAM, single CPU core acceptable).  
   • Uses local filesystem—no external object storage.  
   • No external DB for MVP; use embedded store (e.g., SQLite) or file-based metadata.  
   • Open-source licenses only.

4. High-Level Architecture  
   Backend (Rust)  
     – API layer: REST/JSON endpoints for auth, file CRUD, metadata.  
     – File manager module: abstracts OS I/O, safe path handling, stream-based transfers.  
     – Auth module: user store, session tokens (JWT or signed cookies).  
     – Share module: token generator & validator.  
     – Scheduler: cron-like background tasks.  
     – Config & logging subsystem.

   Frontend (React)  
     – Auth pages: login, logout.  
     – File explorer: folder tree sidebar, main panel list/grid, drag-and-drop area.  
     – Transfer queue component: real-time progress.  
     – Settings dashboard: storage path, user management (future).  
     – Shared-link landing page: minimal file preview & download.

   Data Flow  
     Browser ⇆ HTTPS ⇆ Rust API ⇆ Local filesystem + metadata store

5. Build Phases & Milestones  
   Phase 0 – Project scaffolding  
     • Repo structure, formatting/lint configs, CI.  

   Phase 1 – Core MVP  
     • Auth (single admin)  
     • Directory browsing & basic file CRUD  
     • Storage status indicator  
     • Responsive UI  

   Phase 2 – Quality & Security  
     • HTTPS setup, password hashing, CSRF, rate limiting  
     • Upload resume & cancellation  
     • Background scanner  

   Phase 3 – Sharing & Search  
     • Public link generator  
     • Filename search  

   Phase 4 – Hardening & Packaging  
     • Cross-compilation targets  
     • Install script / Dockerfile  
     • Documentation & test coverage  

   Phase 5 – Stretch Enhancements (optional)  
     • Multiple users & roles  
     • Media streaming (DLNA/HTTP)  
     • Cloud backup hooks

6. Acceptance Criteria  
   • All Phase 1 features run on the target machine with no critical bugs.  
   • Transfers achieve stated performance on Gigabit LAN.  
   • Unit tests ≥80 % coverage for core modules.  
   • CI passes lint, tests, and build for three target platforms.

This specification sets the scope and roadmap. Next step: decide concrete technologies per module and break each phase into implementation tasks.

```

# 2
User:
```
Also concern about meidia service, such as preview of images and display of videos.
And summary this in the @000-design.md  as the spec for the next ai.
```

AI:
Changes are in the @000-design.md.