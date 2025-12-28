# 🔐 rhizoCrypt Live Integration Demos

**Phase 5: Live Primal Integration**

---

## 📋 What You'll Learn

- Starting Phase 1 primals from binaries
- Connecting rhizoCrypt to live services
- Real capability-based discovery
- End-to-end primal communication

---

## 🔧 Prerequisites

The Phase 1 primal binaries must be available at `../../bins/`:

| Binary | Primal | Purpose |
|--------|--------|---------|
| `beardog` | BearDog | Security, signing |
| `songbird-orchestrator` | Songbird | Service discovery |
| `songbird-rendezvous` | Songbird | Rendezvous server |
| `nestgate` | NestGate | Payload storage |
| `toadstool-byob-server` | ToadStool | Compute runtime |

---

## 🚀 Quick Start

```bash
# 1. Start the primal stack
./start-primals.sh

# 2. Run live integration demos
./demo-live-discovery.sh
./demo-live-signing.sh

# 3. Stop the stack when done
./stop-primals.sh
```

---

## 📁 Available Demos

### Infrastructure

| Script | Description |
|--------|-------------|
| `start-primals.sh` | Start BearDog, Songbird, NestGate |
| `stop-primals.sh` | Stop all running primals |
| `check-status.sh` | Check primal health |

### Integration Demos

| Demo | Description | Requires |
|------|-------------|----------|
| `demo-live-discovery.sh` | Real Songbird discovery | Songbird |
| `demo-live-signing.sh` | Real BearDog signing | BearDog |

---

## 🔗 Default Ports

| Primal | Port | Protocol |
|--------|------|----------|
| Songbird Orchestrator | 8080 | HTTP/tarpc |
| Songbird Rendezvous | 8081 | tarpc |
| BearDog | 8091 | HTTP |
| NestGate | 8092 | HTTP |
| ToadStool | 8094 | HTTP |

---

## ⚠️ Notes

- These demos require actual running primals
- Use `check-status.sh` to verify services are healthy
- Logs are written to `/tmp/rhizocrypt-primals/`

---

*rhizoCrypt: Real integration, not just mocks*

