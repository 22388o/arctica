<template>
  <div class="home">
    <img alt="Vue logo" src="../assets/logo.png">
    <HelloWorld msg="Welcome to Your Vue.js App"/>
  </div>
</template>

<script>
// @ is an alias to /src
import { invoke } from '@tauri-apps/api/tauri'
import HelloWorld from '@/components/HelloWorld.vue'

export default {
  name: 'HomeView',
  components: {
    HelloWorld
  },
  mounted() {
  	invoke('my_custom_command', { invokeMessage: 'Hello!' })
  }
}
</script>
