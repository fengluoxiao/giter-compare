<template>
  <div class="about-page">
    <h1>关于 Git Compare Tool</h1>
    <p>这是一个使用 Tauri + Vue 3 开发的 Git 比对工具。</p>
    <p>功能特性：</p>
    <ul>
      <li>行对行比对</li>
      <li>树形文件目录</li>
      <li>实时文件监控</li>
      <li>浅色/深色主题</li>
    </ul>
    <router-link to="/" class="btn btn-primary">返回首页</router-link>
  </div>
</template>

<script setup lang="ts">
// 关于页面逻辑
</script>

<style scoped>
.about-page {
  padding: 40px;
  max-width: 800px;
  margin: 0 auto;
}

.about-page h1 {
  margin-bottom: 20px;
  color: var(--text-primary);
}

.about-page p {
  margin-bottom: 16px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.about-page ul {
  margin-bottom: 24px;
  padding-left: 24px;
}

.about-page li {
  color: var(--text-secondary);
  margin-bottom: 8px;
}
</style>
