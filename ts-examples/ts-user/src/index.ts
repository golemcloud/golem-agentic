import {
    BaseAgent,
    agent,
    prompt,
    description,
} from '@golemcloud/golem-ts-sdk';

import { getSelfMetadata } from 'golem:api/host@1.1.7';

type Question = {
    text: string
}

type LatLong = {lat: number, long: number};

type PlaceName = string;

type Location = LatLong | PlaceName;

@agent()
class AssistantAgent extends BaseAgent {
    @prompt("Ask your question")
    @description("This method allows the agent to answer your question")
    async ask(question: Question): Promise<string> {
        console.log(question);

        console.log(getSelfMetadata().workerId.workerName);

        const location: LatLong = { lat: 12.34, long: 56.78 };

        const remoteWeatherClient = WeatherAgent.get("afsal");
        const remoteWeather = await remoteWeatherClient.getWeather(location);

        return "The weather is: " + remoteWeather;
    }
}

@agent()
class WeatherAgent extends BaseAgent {
    private readonly userName: string;

    constructor(username: string) {
        super()
        this.userName = username;
    }

    @prompt("Get weather")
    @description("Weather forecast weather for you")
    async getWeather(location: Location): Promise<string> {
        return Promise.resolve(
            `Weather for ${location} is sunny!`
        );
    }
}
