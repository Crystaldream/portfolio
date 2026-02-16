---
title: Welcome to My Blog
slug: welcome-to-my-blog
date: 2024-01-15
tags: [welcome, introduction, blog]
published: true
---

# Welcome to My Blog

Hello! Welcome to my personal blog where I share my thoughts on software development, technology, and creative projects.

## What to Expect

Here you'll find:

- **Technical tutorials** - Deep dives into Rust, web development, and system design
- **Project showcases** - Behind the scenes of my personal projects
- **Industry insights** - Thoughts on the tech world and software engineering
- **Learning journeys** - Documenting my exploration of new technologies

## About This Site

This portfolio website is built with a modern Rust stack:

```rust
// The core of the portfolio gateway
let app = Router::new()
    .merge(routes::pages::router())
    .nest("/api", routes::api::router())
    .nest("/htmx", routes::htmx::router())
    .with_state(state);
```

It features:
- Server-side rendering with Askama templates
- HTMX for dynamic interactions without heavy JavaScript
- Custom WASM modules for smooth animations
- SurrealDB for data persistence

## Stay Connected

Feel free to reach out through the [contact page](/contact) or connect with me on social media. I'm always happy to discuss ideas, collaborate on projects, or just chat about tech!

Thanks for visiting!
