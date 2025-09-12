<template>
  <div class="min-h-screen m-0 p-0">
    <Navbar />
    <main>
      <router-view></router-view>
    </main>
    <Footer />
  </div>
</template>

<script setup>
import Navbar from "./components/Navbar.vue";
import Footer from "./components/Footer.vue";
</script>

<style>
/* Add this to ensure no default margins or padding */
body {
  margin: 0;
  padding: 0;
}
</style>
