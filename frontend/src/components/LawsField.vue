<template>
    <!-- Main Board -->
    <v-container class="pa-0 law-container">
        <v-img width="100%" :src="'/img/' + mainImage"></v-img>

        <v-img
            v-for="offset in laws"
            v-bind:key="offset"
            width="12.5%"
            :src="lawImage"
            :style="`position: absolute; top: 26%; left: ${offset}%; z-index: 20;`"
        />
    </v-container>
</template>
<script lang="ts">
import { Faction } from "@/model/types";
import { defineComponent } from "vue";

export default defineComponent({
    name: "BoardView",
    props: {
        mainImage: {
            type: String,
            required: true,
        },
        faction: {
            type: String as () => Faction,
            required: true,
        },
        passed: {
            type: Number,
            required: true,
        },
    },
    computed: {
        laws(): number[] {
            console.log(this.passed);
            const offset = this.faction === "Fasho" ? 9.5 : 16.25;
            return [...Array(this.passed).keys()].map(
                (v) => v * 13.65 + offset,
            );
        },
        lawImage(): string {
            console.log(this.faction);
            return {
                Fasho: "img/law-Fasho.png",
                Liberal: "img/law-Liberal.png",
            }[this.faction];
        },
    },
});
</script>
<style scoped>
.law-container {
    position: relative;
    z-index: 1;
}
</style>
