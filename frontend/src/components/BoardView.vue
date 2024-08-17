<template>
    <!-- 75vw width - padding -->

    <v-row no-gutters>
        <v-col style="border: 1px solid red"> ICON </v-col>
        <v-col>
            <v-spacer />
        </v-col>
        <v-col
            cols="auto"
            class="ml-2"
            v-for="player in state.board.players"
            v-bind:key="player.id"
            style="border: 1px solid blue"
        >
            <PlayerCard :player="player" :state="state" />
        </v-col>
    </v-row>

    <br />

    <v-row justify="end" no-gutters>
        <v-col cols="2" style="border: 1px solid red">
            <v-container width="10vw" style="margin-top: 15px">
                <v-img class="mt-10" src="/img/draw-pile.png" />
            </v-container>
            <v-container width="10vw" style="margin-top: 50px">
                <v-img class="mt-10" src="/img/discard-pile.png" />
            </v-container>
        </v-col>

        <v-col cols="10" width="50vw" style="border: 1px solid green">
            <v-container width="50vw">
                <LawsField
                    :main-image="board_image"
                    :passed="state.board.passed_fasho_laws"
                    faction="Fasho"
                />
            </v-container>

            <v-container width="50vw">
                <LawsField
                    main-image="laws-liberal.png"
                    :passed="state.board.passed_liberal_laws"
                    faction="Liberal"
                />
            </v-container>
        </v-col>
    </v-row>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import { GameStateView } from "@/model/types";
import LawsField from "./LawsField.vue";
import PlayerCard from "./PlayerCard.vue";

export default defineComponent({
    name: "BoardView",
    props: {
        state: {
            type: Object as () => GameStateView,
            required: true,
        },
    },
    computed: {
        board_image(): string {
            return [
                "laws-fasho-6.png",
                "laws-fasho-6.png",
                "laws-fasho-8.png",
                "laws-fasho-8.png",
                "laws-fasho-10.png",
                "laws-fasho-10.png",
            ][Math.max(0, this.state.board.players.length - 5)];
        },
    },
    components: { LawsField, PlayerCard },
});
</script>
