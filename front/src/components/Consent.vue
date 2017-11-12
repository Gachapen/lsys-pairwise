<template lang="pug">
  .consent
    section.consent
      h2 Evaluating Generated Virtual Plants
      p.
        You are being asked to participate in a study investigating ways to improve the process of generating aesthetically pleasing virtual plants that can be used in virtual environment such as games, and be combined into new virtual plants.
      p.
        The program that generates the plants needs to evaluate how aesthetically pleasing the plants are in order to distinguish good plants from bad plants.
        Your contribution in this study will be on evaluating how well this program evaluates the plants.
      p.
        If you agree to participate, you will complete three activities: a pre-questionaire, evaluating pairs of plants, and a post-questionaire.
        The whole process is expected to take around 10 minutes.
      p.
        All data collected is anonymous and confidential.
      p.
        Your participation is voluntary and you may withdraw at any time during the study without prejudice.
      p.
      p
        input(type='checkbox' id='agree' v-model='agree')
        label(for='agree').
          I have read the above and understand the nature of this study.
          I understand that by agreeing to participate in this study I have not waived any legal or human right and that I may contact the researcher (Magnus Bjerke Vik #[a(href='mailto:magnusbv@stud.ntnu.no') &lt;magnusbv@stud.ntnu.no&gt;]) at any time.
          I agree to participate in this study.
          I understand that I may refuse to participate or I may withdraw from the study at any time without prejudice.
      p
        button(:disabled='!agree' @click='$router.push({ path: "register", query: $router.currentRoute.query })') Participate
</template>

<script>
export default {
  data () {
    return {
      agree: false,
    }
  },
}
</script>

<style scoped lang="sass">
</style>
