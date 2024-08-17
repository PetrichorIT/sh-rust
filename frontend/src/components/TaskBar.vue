<template>
    <v-container>
        <template v-if="state.state.type == 'Uninit'">
            <v-btn
                text="Start"
                color="rgb(225, 110, 84)"
                block
                @click="start"
            />
        </template>

        <template v-else-if="task === null">
            <v-row
                class="text-center"
                align="center"
                justify="center"
                style="margin-top: 20vh"
            >
                <v-col>
                    <v-icon size="100" style="opacity: 0.25"
                        >mdi-circle-off-outline</v-icon
                    >
                </v-col>
            </v-row>
        </template>

        <template v-else-if="task!.type === 'ChooseChancellor'">
            <span>
                You are the <b>president</b>. You must choose a
                <b>chancellor canidate</b> that will be put to a vote.
                <br /><br />
                These are your options:
            </span>
            <v-list :selectable="true" v-model:selected="playerListSelection">
                <v-list-item
                    v-for="player of state.board.players"
                    v-bind:key="player.id"
                    :value="player.id"
                    :title="player.user.name"
                    :subtitle="'some roles'"
                    :prepend-avatar="`/img/portraits/${player.user.image}`"
                    :disabled="
                        !(task!.value as ChooseChancellorTask).includes(
                            player.id,
                        )
                    "
                    style="cursor: pointer"
                ></v-list-item>
            </v-list>
            <br />
            <v-btn
                text="Confirm"
                color="rgb(225, 110, 84)"
                block
                :disabled="playerListSelection.length === 0"
                @click="chooseChancellor"
            />
        </template>

        <template v-else-if="task!.type === 'Vote'">
            <span>
                <b>{{ task!.value.president }}</b> has choosen
                <b>{{ task!.value.chancellor }} </b> as his chancellor. You must
                choose whether to approve or deny this election. <br /><br />
            </span>

            <v-row>
                <v-col>
                    <v-img
                        src="/img/yes.png"
                        style="cursor: pointer"
                        :style="
                            voteSelection === true
                                ? 'border: 2px solid rgb(225, 110, 84);'
                                : ''
                        "
                        @click="voteSelection = true"
                    />
                </v-col>
                <v-col>
                    <v-img
                        src="/img/no.png"
                        style="cursor: pointer"
                        :style="
                            voteSelection === false
                                ? 'border: 2px solid rgb(225, 110, 84);'
                                : ''
                        "
                        @click="voteSelection = false"
                    />
                </v-col>
            </v-row>

            <br />
            <v-btn
                text="Confirm"
                color="rgb(225, 110, 84)"
                block
                :disabled="voteSelection === null"
                @click="vote"
            />
        </template>

        <template v-else-if="task!.type === 'PickLaws'">
            <span>
                The following laws have been drawn. Discard one.<br /><br />
            </span>

            <v-list :selectable="true" v-model:selected="selectedLaws">
                <v-list-item
                    v-for="(law, index) of (task!.value as PickLawsTask)[0]"
                    v-bind:key="index"
                    :value="index"
                    :title="law + ' Law'"
                    :prepend-avatar="`/img/law-${law}.png`"
                    style="cursor: pointer"
                ></v-list-item>
            </v-list>

            <br />
            <v-btn
                text="Discard"
                color="rgb(225, 110, 84)"
                block
                :disabled="selectedLaws.length === 0"
                @click="pickLaw"
            />

            <template v-if="(task!.value as PickLawsTask)[1]">
                <br />
                <span>
                    If you want, you can ask the <b>president</b> for a
                    <b>Veto</b> to skip the current legislative session and
                    discards the proposed <b>laws</b>.
                </span>
                <br />
                <br />
                <v-btn
                    text="Ask for Veto"
                    color="rgb(225, 110, 84)"
                    block
                    @click="veto(true)"
                />
            </template>
        </template>

        <template v-else-if="task!.type === 'ConfirmVeto'">
            <span>
                The <b>chancellor</b> has asked for a <b>veto</b>. You can
                either confirm it or deny it.<br /><br />
            </span>

            <v-btn
                text="Confirm"
                color="rgb(225, 110, 84)"
                block
                @click="veto(true)"
            />
            <br />
            <v-btn
                text="Deny"
                color="rgb(225, 110, 84)"
                block
                @click="veto(false)"
            />
        </template>

        <template
            v-else-if="
                task!.type === 'ExecutiveAction' && task!.value?.type === 'Kill'
            "
        >
            <span>
                As an <b>executive action</b> you can kill another player.
                <br /><br />
                These are your options:
            </span>
            <v-list :selectable="true" v-model:selected="playerListSelection">
                <v-list-item
                    v-for="player of state.board.players"
                    v-bind:key="player.id"
                    :value="player.id"
                    :title="player.user.name"
                    :subtitle="'some roles'"
                    :prepend-avatar="`/img/portraits/${player.user.image}`"
                    :disabled="player.id === state.me.id || !player.alive"
                    style="cursor: pointer"
                ></v-list-item>
            </v-list>
            <br />
            <v-btn
                text="Kill"
                color="rgb(225, 110, 84)"
                block
                :disabled="playerListSelection.length === 0"
                @click="killPlayer"
            />
        </template>

        <template
            v-else-if="
                task!.type === 'ExecutiveAction' &&
                task!.value?.type === 'DeterminePresident'
            "
        >
            <span>
                As an <b>executive action</b> you can choose the next
                <b>president</b>. <br /><br />
                These are your options:
            </span>
            <v-list :selectable="true" v-model:selected="playerListSelection">
                <v-list-item
                    v-for="player of state.board.players"
                    v-bind:key="player.id"
                    :value="player.id"
                    :title="player.user.name"
                    :subtitle="'some roles'"
                    :prepend-avatar="`/img/portraits/${player.user.image}`"
                    :disabled="player.id === state.me.id || !player.alive"
                    style="cursor: pointer"
                ></v-list-item>
            </v-list>
            <br />
            <v-btn
                text="Kill"
                color="rgb(225, 110, 84)"
                block
                :disabled="playerListSelection.length === 0"
                @click="specialElection"
            />
        </template>

        <template
            v-else-if="
                task!.type === 'ExecutiveAction' &&
                task!.value?.type === 'RevealNextCards'
            "
        >
            <span>
                As an <b>executive action</b> you can see the next three cards
                in the draw pile. <br /><br />
                These are the next three cards:
            </span>
            <v-list>
                <v-list-item
                    v-for="(law, index) of task!.value!.value"
                    v-bind:key="index"
                    :value="index"
                    :title="law + ' Law'"
                    :prepend-avatar="`/img/law-${law}.png`"
                    style="cursor: pointer"
                ></v-list-item>
            </v-list>
            <br />
            <v-btn
                text="Ok"
                color="rgb(225, 110, 84)"
                block
                @click="revealCards"
            />
        </template>

        <template
            v-else-if="
                task!.type === 'ExecutiveAction' &&
                task!.value?.type === 'RevealFaction'
            "
        >
            <span>
                As an <b>executive action</b> you can learn the
                <b>faction</b> of another player. <br /><br />
                Choose a playerr:
            </span>
            <v-list :selectable="true" v-model:selected="playerListSelection">
                <v-list-item
                    v-for="player of state.board.players"
                    v-bind:key="player.id"
                    :value="player.id"
                    :title="player.user.name"
                    :subtitle="'some roles'"
                    :prepend-avatar="`/img/portraits/${player.user.image}`"
                    :disabled="player.id === state.me.id || !player.alive"
                    style="cursor: pointer"
                ></v-list-item>
            </v-list>
            <br />
            <v-btn
                text="Reveal"
                color="rgb(225, 110, 84)"
                block
                :disabled="playerListSelection.length === 0"
                @click="revealFaction"
            />
        </template>

        <template v-else> {{ task }} </template>
    </v-container>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import {
    ChooseChancellorTask,
    Faction,
    GameStateView,
    PickLawsTask,
    PlayerId,
    Task,
} from "src/model/types";

export type TaskOrNull = Task | null;

export default defineComponent({
    name: "TaskBar",
    emits: ["action"],
    data() {
        return {
            playerListSelection: [] as PlayerId[],
            voteSelection: null as null | boolean,
            selectedLaws: [] as number[],
        };
    },
    props: {
        task: {
            type: Object as () => TaskOrNull,
            required: false,
        },
        state: {
            type: Object as () => GameStateView,
            required: true,
        },
    },
    methods: {
        start() {
            this.$emit("action", { type: "Start" });
        },
        chooseChancellor() {
            console.log(this.playerListSelection);
            this.$emit("action", {
                type: "ChooseChancellor",
                value: this.playerListSelection[0],
            });
        },
        vote() {
            console.log(this.playerListSelection);
            this.$emit("action", {
                type: "Vote",
                value: this.voteSelection!,
            });
        },
        pickLaw() {
            console.log(this.selectedLaws);
            let idx = this.selectedLaws[0];
            let laws: Faction[] = JSON.parse(
                JSON.stringify(this.task!.value[0]),
            );
            let discarded = laws.splice(idx, 1)[0];
            this.$emit("action", {
                type: "PickedLaws",
                value: [laws, discarded],
            });
        },
        killPlayer() {
            console.log(this.playerListSelection);
            this.$emit("action", {
                type: "ExecuteAction",
                value: {
                    type: "Kill",
                    value: this.playerListSelection[0],
                },
            });
        },
        revealCards() {
            console.log(this.playerListSelection);
            this.$emit("action", {
                type: "ExecuteAction",
                value: { type: "RevealNextCards" },
            });
        },
        specialElection() {
            console.log(this.playerListSelection);
            this.$emit("action", {
                type: "ExecuteAction",
                value: {
                    type: "DeterminePresident",
                    value: this.playerListSelection[0],
                },
            });
        },
        revealFaction() {
            console.log(this.playerListSelection);
            this.$emit("action", {
                type: "ExecuteAction",
                value: {
                    type: "RevealFaction",
                    value: this.playerListSelection[0],
                },
            });
        },
        veto(accept: boolean) {
            this.$emit("action", {
                type: "Veto",
                value: accept,
            });
        },
    },
});
</script>
<style scoped>
.playerContainer {
    border-radius: 5px;
    text-align: center;
    z-index: 99;
}

.activeSelectOption {
    cursor: pointer;
}
</style>
