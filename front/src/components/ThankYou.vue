<template lang="pug">
  .thanks
    section
      .confetti 🎊
      h1 Thank you for participating!
      .links
        .link.share(v-if='task && publicToken')
          i.fa.fa-share-alt
          router-link(:to='`/?task=${task}&from=${publicToken}`') Share this link to let others participate.
        .link
          i.fa.fa-bookmark
          router-link(:to='`/result/${token}?no-questions=true`') Save this link to see your results again!
        p(v-if='mturk') Your mturk code: {{ publicToken }}
</template>

<script>
import { get } from 'axios'
import screenfull from 'screenfull'
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
      publicToken: null,
      mturk: false,
    }
  },
  created () {
    if (screenfull.enabled && screenfull.isFullscreen) {
      screenfull.exit()
    }

    get(`${API_BASE}/user/${this.token}/task`)
      .then(response => {
        this.task = response.data.task
      })
      .catch(error => console.error('Failed retrieving user task', error))

    get(`${API_BASE}/user/${this.token}/public`)
      .then(response => {
        this.publicToken = response.data.public
      })
      .catch(error => console.error('Failed retrieving public token', error))

    get(`${API_BASE}/user/${this.token}/source`)
      .then(response => {
        this.mturk = response.data.source === 'mturk'
      })
      .catch(error => console.error('Failed retrieving public token', error))
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

.confetti
  font-size: 200px
</style>
