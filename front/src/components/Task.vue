<template lang="pug">
  .pairwise
    component(is='comparison' :pair='currentPair' :key='pairId' :realistic.sync='realistic' :pleasing.sync='pleasing')
    button.next(v-if='!isLast' @click='next' :disabled='!canContinue') Next
    button.finish(v-else @click='finish' :disabled='!canContinue') Finish
</template>

<script>
import Comparison from './Comparison'
import { API_BASE } from '../config'
import axios, { get, post } from 'axios'

export default {
  components: {
    Comparison,
  },
  props: {
    token: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
      pairs: [],
      pairIndex: 0,
      realistic: undefined,
      pleasing: undefined,
    }
  },
  computed: {
    currentPair () {
      if (this.pairIndex < this.pairs.length) {
        return this.pairs[this.pairIndex]
      } else {
        return {
          a: undefined,
          b: undefined,
        }
      }
    },
    pairId () {
      return `${this.currentPair.a} ${this.currentPair.b}`
    },
    canContinue () {
      return this.realistic && this.pleasing
    },
    isLast () {
      return this.pairIndex === this.pairs.length - 1
    },
  },
  methods: {
    postWeight (metric, weight) {
      return post(`${API_BASE}/weight`, {
        token: this.token,
        metric,
        a: this.currentPair.a,
        b: this.currentPair.b,
        weight,
      })
    },
    next () {
      axios.all([
        this.postWeight('realistic', this.realistic),
        this.postWeight('pleasing', this.pleasing),
      ])
        .then(() => {
          this.pairIndex += 1
          this.realistic = undefined
          this.pleasing = undefined
        })
        .catch(error => console.error('Failed posting weights', error))
    },
    finish () {
      axios.all([
        this.postWeight('realistic', this.realistic),
        this.postWeight('pleasing', this.pleasing),
      ])
        .then(() => {
          this.$router.push({ path: '/result', params: { token: this.token } })
        })
        .catch(error => console.error('Failed posting weights', error))
    },
  },
  created () {
    get(`${API_BASE}/task`)
      .then(response => {
        this.pairs = response.data
        this.pairIndex = 0
      })
      .catch(error => console.error('Failed retrieving task', error))
  },
}
</script>

<style scoped lang="sass">
.pairwise
  width: 100%
  height: 100%
  padding-top: 20px
  background-color: rgb(127, 127, 127)

  >button
    margin-top: 10px
    padding: 10px 20px
    border: 0
    font-size: 12pt
    background: rgb(200, 200, 200)
</style>
