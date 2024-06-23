# Best Practices

It is important to keep some best practices in mind when making a script, so that things continue to work properly.

Please ensure you follow these tips carefully.

```admonish warning title="Code with API changes in mind" collapsible=true
Api may change between plugin versions as this product is updated.

Because of that, it is _very important_ to script against specific versions of native memory scripter so you can account for any api changes and keep things running smoothly with no errors on the user side. Using this technique, you can support multiple versions seamlessly. You can check the plugin version in your script by using [info.version](./info/version.md). All api changes are catalogued in the changelog.
```

```admonish warning title="Beware of Dropping" collapsible=true
Objects that require allocation such as [`WStr`](./cffi/objects-wstr.md), [`Callable`](./cffi/objects-callable.md), and others, automatically free their memory when deleted or reclaimed by gc. This can serve as a source of hidden UB since it can cause a UAF. Because of this, even allocated code can suddenly disappear during execution!

Make sure that any allocated objects in Python stay alive during the duration you're using them. This is absolutely crucial. This especially applies when returning some allocated data from a callback function. If the data goes out of scope, python reclaims it in gc, so your returned data is now UAF.

If you need to debug if something is dropping prematurely, turning on trace logs which will help you determine when drops are happening.
```
