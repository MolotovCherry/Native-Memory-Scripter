# Best Practices

It is important to keep some best practices in mind when making a script, so that things continue to work properly.

Please ensure you follow these tips carefully.

```admonish warning title="Code with API changes in mind"
Api may change between plugin versions as this product is updated.

Because of that, it is _very important_ to script against specific versions of native memory scripter so you can account for any api changes and keep things running smoothly with no errors on the user side. Using this technique, you can support multiple versions seamlessly. You can check the plugin version in your script by using [info.version](./info/version.md). All api changes are catalogued in the changelog.
```
