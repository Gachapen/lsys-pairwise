<template lang="pug">
  .comparison-slider
    template(v-for='point in points')
      label
        input(type='radio' :name='equal')
        | {{ point.label }}
    //- .point(v-for='point in points')
    //-   .shape
    //-   .label {{ point.label }}
</template>

<script>
export default {
  props: {
    more: {
      type: String,
      required: true,
    },
    equal: {
      type: String,
      required: true,
    },
  },
  data () {
    return {
    }
  },
  computed: {
    points () {
      return [
        {
          weight: 1 / 9,
          label: `Extremely ${this.more}`,
        },
        {
          weight: 1 / 7,
          label: `Far ${this.more}`,
        },
        {
          weight: 1 / 5,
          label: `Much ${this.more}`,
        },
        {
          weight: 1 / 3,
          label: `Slightly ${this.more}`,
        },
        {
          weight: 1,
          label: `Equally ${this.equal}`,
        },
        {
          weight: 3,
          label: `Slightly ${this.more}`,
        },
        {
          weight: 5,
          label: `Much ${this.more}`,
        },
        {
          weight: 7,
          label: `Far ${this.more}`,
        },
        {
          weight: 9,
          label: `Extremely ${this.more}`,
        },
      ]
    },
  },
}
</script>

<style scoped lang="sass">
.comparison-slider
  >label
    display: inline-block
    width: 80px
    vertical-align: top
    margin: 0 30px

    &:first-child
      margin-left: 0px

    &:last-child
      margin-right: 0px

    >input
      display: block
      margin: 0 auto
      margin-bottom: 5px
</style>
