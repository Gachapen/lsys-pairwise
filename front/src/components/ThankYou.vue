<template lang="pug">
  .thanks
    section
      h1 Thank you for participating!
      .links
        .link
          i.fa.fa-bookmark
          router-link(:to='`/result/${token}?no-questions=true`') Save this link to see your ranking again!
        .link.share(v-if='task')
          i.fa.fa-share-alt
          router-link(:to='`/?task=${task}`') Share this link to let others participate.
</template>

<script>
import { get } from 'axios'
import { API_BASE } from '../config'

export default {
  props: {
    token: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
      task: null,
    }
  },
  created () {
    get(`${API_BASE}/user/${this.token}/task`)
      .then(response => {
        this.task = response.data.task
      })
      .catch(error => console.error('Failed retrieving user task', error))
  },
}
</script>

<style scoped lang="sass">
section
  text-align: center

.links
  display: inline-block
  text-align: left

.link
  margin-top: 15px
  >i.fa
    margin-right: 10px

.link.share > a
  font-weight: bold
  font-size: 14pt
</style>
