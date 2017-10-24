<template lang="pug">
  .pairwise
    component(is='comparison', :pair='currentPair', :key='pairId')
    button.next(@click='next') Next
</template>

<script>
import Comparison from './Comparison'
import { API_BASE } from '../config'
import { get } from 'axios'

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
  },
  methods: {
    next () {
      this.pairIndex += 1
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

  >button.next
    margin-top: 10px
    padding: 10px 20px
    border: 0
    font-size: 12pt
    background: rgb(200, 200, 200)
</style>
