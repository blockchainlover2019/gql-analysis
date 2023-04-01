const { ApolloRouter } = require("@apollo/router");
const { readFileSync } = require("fs");

async function start() {
  const router = new ApolloRouter();

  const configString = readFileSync("./router.config.yaml", "utf-8");
  await router.loadConfigFromString(configString);

  const server = await router.listen({ port: 4000 });

  console.log(`🚀 Apollo Router ready at http://localhost:4000/graphql`);
}

start().catch((error) => {
  console.error("Failed to start Apollo Router:", error);
  process.exit(1);
});