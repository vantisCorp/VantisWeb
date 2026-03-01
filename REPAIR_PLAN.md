# Plan Naprawy Projektu VantisWeb Browser

## 🎯 Cel: Doprowadzenie projektu do stanu kompilującego się (0 błędów)

---

## Faza 1: Naprawa Krytycznych Błędów Kompilacji (2-3 godziny)

### Krok 1.1: Naprawa eksportów w src/engine/mod.rs
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🔴 KRYTYCZNY  
**Szacowany czas:** 15 minut

**Zadania:**
- [ ] Dodaj brakujące eksporty dla `event_loop::{Event, EventHandler, EventLoop, Task, TaskType, TaskStatus, EventLoopStats}`
- [ ] Dodaj brakujące eksporty dla `fetch::{FetchApi, HttpRequest, HttpResponse, HttpMethod, HttpHeaders, RequestBody, CorsMode}`
- [ ] Dodaj brakujące eksporty dla `storage::{StorageApi, StorageType, StorageEvent, StorageEntry, CookieManager, Cookie}`
- [ ] Dodaj brakujące eksporty dla `console::{ConsoleApi, LogLevel, ConsoleEntry, PerformanceMetric}`

**Plik do modyfikacji:** `src/engine/mod.rs`

---

### Krok 1.2: Usunięcie HTML entities z kodu źródłowego
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🔴 KRYTYCZNY  
**Szacowany czas:** 30 minut

**Zadania:**
- [ ] Znajdź wszystkie wystąpienia `&amp;` w plikach `.rs`
- [ ] Zastąp `&amp;` przez `&`
- [ ] Znajdź wszystkie wystąpienia `&lt;` w plikach `.rs`
- [ ] Zastąp `&lt;` przez `<`
- [ ] Znajdź wszystkie wystąpienia `&gt;` w plikach `.rs`
- [ ] Zastąp `&gt;` przez `>`
- [ ] Znajdź wszystkie wystąpienia `&quot;` w plikach `.rs`
- [ ] Zastąp `&quot;` przez `"`

**Pliki do modyfikacji:** Wszystkie pliki `.rs` w `src/`

---

### Krok 1.3: Naprawa błędów lifetime
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🔴 KRYTYCZNY  
**Szacowany czas:** 45 minut

**Zadania:**
- [ ] Znajdź wszystkie błędy `E0106` (missing lifetime specifier)
- [ ] Dodaj brakujące specyfikatory lifetime
- [ ] Napraw referencje w strukturach
- [ ] Sprawdź czy wszystkie struktury poprawnie implementują wymagane traity

**Pliki do modyfikacji:** `src/engine/*.rs`

---

### Krok 1.4: Weryfikacja kompilacji
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🔴 KRYTYCZNY  
**Szacowany czas:** 15 minut

**Zadania:**
- [ ] Uruchom `cargo check`
- [ ] Sprawdź czy liczba błędów wynosi 0
- [ ] Sprawdź czy liczba ostrzeżeń jest < 10
- [ ] Jeśli są błędy, wróć do poprzednich kroków

---

### Krok 1.5: Uruchomienie testów
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 15 minut

**Zadania:**
- [ ] Uruchom `cargo test`
- [ ] Sprawdź czy wszystkie testy przechodzą
- [ ] Jeśli testy nie przechodzą, napraw je

---

## Faza 2: Uzupełnienie Implementacji Web Engine (20-30 godzin)

### Krok 2.1: WebRenderer - Integracja z WebKitGTK
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 5 godzin

**Zadania:**
- [ ] Dodaj zależność `webkit2gtk` do `Cargo.toml`
- [ ] Zaimplementuj integrację z WebKitGTK
- [ ] Dodaj obsługę renderowania HTML5
- [ ] Dodaj obsługę renderowania CSS3
- [ ] Dodaj obsługę JavaScript
- [ ] Dodaj obsługę zdarzeń
- [ ] Dodaj obsługę nawigacji
- [ ] Dodaj obsługę historii

**Plik do modyfikacji:** `src/engine/web_renderer.rs`

---

### Krok 2.2: DOM Manager - Pełna obsługa HTML5
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 4 godziny

**Zadania:**
- [ ] Zaimplementuj pełną obsługę HTML5
- [ ] Dodaj obsługę zdarzeń DOM
- [ ] Dodaj manipulację DOM
- [ ] Dodaj query selectors
- [ ] Dodaj obsługę atrybutów
- [ ] Dodaj obsługę stylów

**Plik do modyfikacji:** `src/engine/dom.rs`

---

### Krok 2.3: JavaScript Runtime - Integracja z JavaScriptCore
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 5 godzin

**Zadania:**
- [ ] Dodaj zależność `javascriptcore-rs` do `Cargo.toml`
- [ ] Zaimplementuj integrację z JavaScriptCore
- [ ] Dodaj obsługę ES6+
- [ ] Dodaj obsługę async/await
- [ ] Dodaj obsługę Promise
- [ ] Dodaj debugging
- [ ] Dodaj optymalizację wydajności

**Plik do modyfikacji:** `src/engine/js_runtime.rs`

---

### Krok 2.4: WebAssembly - Runtime WASM
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟠 NORMALNE  
**Szacowany czas:** 3 godziny

**Zadania:**
- [ ] Dodaj zależność `wasmtime` do `Cargo.toml`
- [ ] Zaimplementuj runtime WASM
- [ ] Dodaj obsługę WASI
- [ ] Dodaj optymalizację
- [ ] Dodaj obsługę modułów

**Plik do modyfikacji:** `src/engine/wasm.rs`

---

### Krok 2.5: Navigation System - Forward/back navigation
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 3 godziny

**Zadania:**
- [ ] Zaimplementuj forward/back navigation
- [ ] Dodaj History API
- [ ] Dodaj page lifecycle
- [ ] Dodaj obsługę błędów nawigacji
- [ ] Dodaj obsługę redirectów

**Plik do modyfikacji:** `src/engine/navigation.rs`

---

## Faza 3: Implementacja Modułów Placeholder (15-20 godzin)

### Krok 3.1: AI Module - Podstawowa struktura
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟠 NORMALNE  
**Szacowany czas:** 5 godzin

**Zadania:**
- [ ] Utwórz strukturę modułu AI
- [ ] Dodaj zależność `tch-rs` do `Cargo.toml`
- [ ] Zaimplementuj podstawowe funkcje AI
- [ ] Dodaj obsługę modeli ML
- [ ] Dodaj obsługę inferencji
- [ ] Dodaj testy

**Plik do utworzenia:** `src/ai/mod.rs`, `src/ai/core.rs`, `src/ai/models.rs`

---

### Krok 3.2: Network Module - Obsługa HTTP/HTTPS
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟠 NORMALNE  
**Szacowany czas:** 5 godzin

**Zadania:**
- [ ] Utwórz strukturę modułu Network
- [ ] Zaimplementuj obsługę HTTP/HTTPS
- [ ] Dodaj obsługę WebSocket
- [ ] Dodaj obsługę Tor (placeholder)
- [ ] Dodaj testy

**Plik do utworzenia:** `src/network/mod.rs`, `src/network/http.rs`, `src/network/websocket.rs`

---

### Krok 3.3: Profiles Module - Zarządzanie profilami
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟠 NORMALNE  
**Szacowany czas:** 5 godzin

**Zadania:**
- [ ] Utwórz strukturę modułu Profiles
- [ ] Zaimplementuj tworzenie profili
- [ ] Zaimplementuj zarządzanie profilami
- [ ] Dodaj izolację profili
- [ ] Dodaj testy

**Plik do utworzenia:** `src/profiles/mod.rs`, `src/profiles/manager.rs`, `src/profiles/profile.rs`

---

### Krok 3.4: Modules Module - System rozszerzeń
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟠 NORMALNE  
**Szacowany czas:** 5 godzin

**Zadania:**
- [ ] Utwórz strukturę modułu Modules
- [ ] Zaimplementuj system ładowania modułów
- [ ] Zaimplementuj API rozszerzeń
- [ ] Dodaj sandbox dla rozszerzeń
- [ ] Dodaj testy

**Plik do utworzenia:** `src/modules/mod.rs`, `src/modules/loader.rs`, `src/modules/extension.rs`

---

## Faza 4: Testowanie i Optymalizacja (10-15 godzin)

### Krok 4.1: Testy jednostkowe
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 5 godzin

**Zadania:**
- [ ] Dodaj testy dla Web Engine
- [ ] Dodaj testy dla AI Module
- [ ] Dodaj testy dla Network Module
- [ ] Dodaj testy dla Profiles Module
- [ ] Dodaj testy dla Modules Module
- [ ] Sprawdź pokrycie kodu testami (>70%)

**Pliki do utworzenia:** `tests/engine_tests.rs`, `tests/ai_tests.rs`, `tests/network_tests.rs`, `tests/profiles_tests.rs`, `tests/modules_tests.rs`

---

### Krok 4.2: Testy integracyjne
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 3 godziny

**Zadania:**
- [ ] Dodaj testy integracji modułów
- [ ] Dodaj testy end-to-end
- [ ] Dodaj testy wydajności
- [ ] Sprawdź czy wszystkie testy przechodzą

**Plik do utworzenia:** `tests/integration_tests.rs`

---

### Krok 4.3: Optymalizacja
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟠 NORMALNE  
**Szacowany czas:** 2 godziny

**Zadania:**
- [ ] Optymalizacja pamięci
- [ ] Optymalizacja CPU
- [ ] Optymalizacja renderowania
- [ ] Profilowanie kodu
- [ ] Usunięcie niepotrzebnego kodu

---

## Faza 5: Dokumentacja i Release (5-10 godzin)

### Krok 5.1: Dokumentacja API
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🔵 POŻĄDANE  
**Szacowany czas:** 3 godziny

**Zadania:**
- [ ] Dodaj dokumentację Web Engine
- [ ] Dodaj dokumentację AI Module
- [ ] Dodaj dokumentację Network Module
- [ ] Dodaj dokumentację Profiles Module
- [ ] Dodaj dokumentację Modules Module
- [ ] Sprawdź czy dokumentacja jest kompletna (>95%)

**Plik do utworzenia:** `docs/WEB_ENGINE_API.md`, `docs/AI_API.md`, `docs/NETWORK_API.md`, `docs/PROFILES_API.md`, `docs/MODULES_API.md`

---

### Krok 5.2: Przykłady użycia
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🔵 POŻĄDANE  
**Szacowany czas:** 2 godziny

**Zadania:**
- [ ] Dodaj przykłady kodu
- [ ] Dodaj tutoriale
- [ ] Dodaj przewodniki
- [ ] Dodaj FAQ

**Plik do utworzenia:** `examples/`, `docs/TUTORIALS.md`, `docs/FAQ.md`

---

### Krok 5.3: Release v0.2.0
**Status:** ⏳ Do zrobienia  
**Priorytet:** 🟡 WAŻNE  
**Szacowany czas:** 1 godzina

**Zadania:**
- [ ] Aktualizacja CHANGELOG.md
- [ ] Aktualizacja Cargo.toml (wersja 0.2.0)
- [ ] Utworzenie tagu v0.2.0
- [ ] Publikacja release na GitHub
- [ ] Aktualizacja ROADMAP.md

---

## 📊 Podsumowanie Planu

### Czas realizacji:
- **Faza 1:** 2-3 godziny
- **Faza 2:** 20-30 godzin
- **Faza 3:** 15-20 godzin
- **Faza 4:** 10-15 godzin
- **Faza 5:** 5-10 godzin
- **Razem:** 52-78 godzin (~7-10 dni roboczych)

### Priorytety:
- 🔴 KRYTYCZNE: Faza 1 (musi być zrobiona natychmiast)
- 🟡 WAŻNE: Faza 2, 4.1, 4.2, 5.3 (powinna być zrobiona wkrótce)
- 🟠 NORMALNE: Faza 3, 4.3 (powinna być zrobiona)
- 🔵 POŻĄDANE: Faza 5.1, 5.2 (można zrobić później)

### Cele:
- **Krótkoterminowe (1-2 tygodnie):** Faza 1 + Faza 2
- **Średnioterminowe (1-2 miesiące):** Faza 3 + Faza 4
- **Długoterminowe (3-6 miesięcy):** Faza 5

---

## 🚀 Szybki Start

Jeśli chcesz szybko naprawić błędy kompilacji, wykonaj tylko:

1. **Krok 1.1:** Naprawa eksportów w `src/engine/mod.rs` (15 minut)
2. **Krok 1.2:** Usunięcie HTML entities z kodu (30 minut)
3. **Krok 1.3:** Naprawa błędów lifetime (45 minut)
4. **Krok 1.4:** Weryfikacja kompilacji (15 minut)

**Razem:** ~2 godziny

Po wykonaniu tych kroków projekt powinien się kompilować bez błędów!

---

*Plan przygotowany przez SuperNinja*  
*Data: 2026-03-01*