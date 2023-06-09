defmodule Editor.MixProject do
  use Mix.Project

  def project do
    [
      app: :editor,
      version: "0.1.0",
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      releases: releases()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {Editor.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      # {:dep_from_hexpm, "~> 0.3.0"},
      # {:dep_from_git, git: "https://github.com/elixir-lang/my_dep.git", tag: "0.1.0"}
      {:rustler, "~> 0.27.0"},
      {:mix_test_watch, "~> 1.1", only: :dev, runtime: false},
      {:jason, "~> 1.4"},
      {:burrito, github: "burrito-elixir/burrito"}
    ]
  end

  defp releases do
    [
      editor: [
        steps: [:assemble, &Burrito.wrap/1],
        burrito: [
          targets: [
            macos: [os: :darwin, cpu: :aarch64]
          ]
        ]
      ]
    ]
  end
end
