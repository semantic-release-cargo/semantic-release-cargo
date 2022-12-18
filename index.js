const { platform, arch } = require("os");

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
  semanticReleaseCargo.verifyConditions();
}

function prepare(pluginConfig, context) {
  semanticReleaseCargo.prepare(context.nextRelease.version);
}

function publish(pluginConfig, context) {
  semanticReleaseCargo.publish(false);
}

module.exports = {
  verifyConditions,
  prepare,
  publish,
};
