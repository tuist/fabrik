---
layout: page
title: Blog
description: Latest updates, announcements, and insights about Fabrik
---

<script setup>
import { data as posts } from '../.vitepress/theme/posts.data.mts'
</script>

# Blog

Latest updates and insights about Fabrik's development and features.

<div class="blog-posts">
  <article v-for="post in posts" :key="post.url" class="post-item">
    <h2><a :href="post.url">{{ post.title }}</a></h2>
    <div class="post-meta">
      <time :datetime="post.date">{{ new Date(post.date).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' }) }}</time>
      <span v-if="post.author"> by {{ post.author }}</span>
    </div>
    <p>{{ post.description }}</p>
    <div class="post-tags" v-if="post.tags && post.tags.length">
      <span v-for="tag in post.tags" :key="tag" class="tag">{{ tag }}</span>
    </div>
  </article>
</div>

<style scoped>
.blog-posts {
  margin-top: 2rem;
}

.post-item {
  margin-bottom: 3rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--vp-c-divider);
}

.post-item:last-child {
  border-bottom: none;
}

.post-item h2 {
  margin: 0 0 0.5rem 0;
  border-top: none;
}

.post-item h2 a {
  color: var(--vp-c-brand-1);
  text-decoration: none;
}

.post-item h2 a:hover {
  text-decoration: underline;
}

.post-meta {
  color: var(--vp-c-text-2);
  font-size: 0.9rem;
  margin-bottom: 0.75rem;
}

.post-meta time {
  font-weight: 500;
}

.post-item p {
  margin: 0 0 1rem 0;
  line-height: 1.6;
}

.post-tags {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.tag {
  background: var(--vp-c-default-soft);
  color: var(--vp-c-text-2);
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.85rem;
}
</style>
