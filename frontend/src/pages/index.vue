<template>
    <v-row no-gutters>
        <v-col cols="9">
            <v-container>
                <BoardView v-if="state !== null" :state="state" />
            </v-container>
        </v-col>
        <v-col cols="3">
            <v-card
                class="ma-2"
                height="95vh"
                style="
                    border: 15px solid rgb(243, 226, 198);
                    border-radius: 15px;
                "
            >
                <v-card-text>
                    <v-tabs v-model="tab" align-tabs="center" stacked>
                        <v-tab v-if="!auth" value="login">
                            <v-icon icon="mdi-login"></v-icon>Login
                        </v-tab>
                        <v-tab value="actions">
                            <v-icon icon="mdi-home"></v-icon>Actions
                        </v-tab>
                        <v-tab value="history">
                            <v-icon icon="mdi-chat"></v-icon>History
                        </v-tab>
                    </v-tabs>

                    <v-tabs-window v-model="tab">
                        <v-tabs-window-item v-if="!auth" value="login">
                            <v-container>
                                <span>
                                    Login as a user:
                                    <br /><br />
                                </span>
                                <v-text-field
                                    v-model="loginName"
                                    label="Username"
                                    density="compact"
                                />
                                <v-select
                                    v-model="loginImage"
                                    label="Profile Image"
                                    density="compact"
                                    :items="[
                                        'p0.png',
                                        'p1.png',
                                        'p2.png',
                                        'p3.png',
                                        'p4.png',
                                        'p5.png',
                                        'p6.png',
                                        'p7.png',
                                        'p8.png',
                                        'p9.png',
                                    ]"
                                />
                                <v-color-picker
                                    v-model="loginColor"
                                    label="Theme color"
                                    hide-canvas
                                    width="100%"
                                ></v-color-picker>

                                <br />
                                <v-btn
                                    text="Login"
                                    color="rgb(225, 110, 84)"
                                    block
                                    @click="login"
                                />
                            </v-container>
                        </v-tabs-window-item>

                        <v-tabs-window-item value="actions">
                            <TaskBar
                                v-if="state !== null"
                                :state="state!"
                                :task="task"
                                @action="runAction"
                            />
                        </v-tabs-window-item>

                        <v-tabs-window-item value="history">
                            <HistoryList
                                v-if="state !== null"
                                :state="state!"
                            />
                        </v-tabs-window-item>
                    </v-tabs-window>
                </v-card-text>
            </v-card>
        </v-col>
    </v-row>
</template>

<script lang="ts">
import BoardView from "@/components/BoardView.vue";
import HistoryList from "@/components/HistoryList.vue";
import TaskBar from "@/components/TaskBar.vue";
import { connect, Connection } from "@/model/connection";
import { GameStateView, Task, TaskAction, User } from "@/model/types";
import { defineComponent } from "vue";

export default defineComponent({
    name: "GamePage",
    components: { BoardView, TaskBar, HistoryList },
    data() {
        return {
            tab: "login",
            con: null as Connection | null,
            state: null as null | GameStateView,
            task: null as null | Task,
            auth: false,

            loginName: `U${Math.random().toString().substring(2, 8)}`,
            loginImage: `p${Math.floor(Math.random() * 9)}.png`,
            loginColor: "purple",
        };
    },
    methods: {
        runAction(action: TaskAction) {
            this.con?.send(action);
        },
        login() {
            const user: User = {
                name: this.loginName,
                image: this.loginImage,
                color: this.loginColor,
            };
            this.con?.login(user);
        },
    },
    beforeMount() {
        this.con = connect((state, task, auth) => {
            console.log("rerender");
            this.state = state;
            this.task = task;
            this.auth = auth;
            if (this.auth && this.tab === "login") {
                this.tab = "actions";
            }
            this.$forceUpdate();
        });
    },
});
</script>
