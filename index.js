const { platform, arch } = require("node:os");
const path = require("node:path");

const getAbi = (platform) => {
  switch (platform) {
    case "linux":
      return "-gnu";
    case "win32":
      return "-msvc";
    default:
      return "";
  }
};

const MODULE_NAME = "semantic-release-cargo";
const PLATFORM = platform();
const ARCH = arch();
const ABI = getAbi(PLATFORM);
const semanticReleaseCargo = require(`./napi/${MODULE_NAME}.${PLATFORM}-${ARCH}${ABI}.node`);

function verifyConditions(pluginConfig, context) {
  semanticReleaseCargo.verifyConditions(pluginConfig);
}

function prepare(pluginConfig, context) {
  semanticReleaseCargo.prepare(context.nextRelease.version, pluginConfig);
}

function publish(pluginConfig, context) {
  semanticReleaseCargo.publish(pluginConfig);
}

module.exports = {
  verifyConditions,
  prepare,
  publish,
};
