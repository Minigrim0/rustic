"""
Interactive IPython widget helpers for MLflow run management.

Provides one-click widgets for registering models and annotating runs,
as well as utilities for listing registered models from the MLflow registry.
Intended for use inside Jupyter notebooks.
"""


def fetch_registered_models(tracking_uri: str | None = None) -> list:
    """Return all registered models from the MLflow registry.

    Args:
        tracking_uri: MLflow tracking server URI. Uses the active URI if None.

    Returns:
        List of RegisteredModel objects.
    """
    import mlflow
    from mlflow.tracking import MlflowClient

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)
    return MlflowClient().search_registered_models()


def display_registered_models(models: list) -> None:
    """Print a summary table of registered models and their versions.

    Args:
        models: List of RegisteredModel objects as returned by
                :func:`fetch_registered_models`.
    """
    from mlflow.tracking import MlflowClient

    if not models:
        print("No registered models found.")
        return

    client = MlflowClient()
    for rm in models:
        versions = client.search_model_versions(f"name='{rm.name}'")
        if not versions:
            print(f"{rm.name}  (no versions)")
            continue
        latest = max(versions, key=lambda v: int(v.version))
        print(f"\n{rm.name}  —  {len(versions)} version(s), latest: v{latest.version}")
        print(f"  {'Ver':<6} {'Run ID':<36} {'Aliases'}")
        print(f"  {'-'*6} {'-'*36} {'-'*20}")
        for v in sorted(versions, key=lambda v: int(v.version), reverse=True):
            aliases = ", ".join(v.aliases) if v.aliases else "-"
            print(f"  v{v.version:<5} {v.run_id}  {aliases}")


def show_registered_models(tracking_uri: str | None = None) -> None:
    """Print a summary of all registered models and their versions.

    Convenience wrapper around :func:`fetch_registered_models` and
    :func:`display_registered_models`.

    Args:
        tracking_uri: MLflow tracking server URI. Uses the active URI if None.
    """
    display_registered_models(fetch_registered_models(tracking_uri))


def setup_mlflow(tracking_uri: str, experiment_name: str) -> None:
    """Set the MLflow tracking URI and active experiment.

    Call this once per notebook before any mlflow.start_run() calls.

    Args:
        tracking_uri:    MLflow server URI, e.g. "http://192.168.1.254:5000".
        experiment_name: Name of the experiment to log runs under. Created
                         automatically if it does not exist yet.
    """
    import mlflow

    mlflow.set_tracking_uri(tracking_uri)
    mlflow.set_experiment(experiment_name)
    print(f"MLflow connected: {tracking_uri}  (experiment: {experiment_name})")


def show_register_widget(
    run_id: str,
    model_artifact: str = "model",
    default_name: str = "",
    tracking_uri: str | None = None,
) -> None:
    """Display a text field + button to register the model logged in a run.

    Args:
        run_id:          MLflow run ID whose logged model will be registered.
        model_artifact:  Artifact path used when logging the model
                         (e.g. "note_discriminator" or "adsr_predictor").
                         Defaults to "model".
        default_name:    Pre-filled registered model name in the text field.
                         Leave empty to force the user to type one.
        tracking_uri:    MLflow tracking server URI. Uses the active URI if None.
    """
    import mlflow
    import ipywidgets as widgets
    from IPython.display import display

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    name_input = widgets.Text(
        value=default_name,
        placeholder="Registered model name",
        description="Name:",
        layout=widgets.Layout(width="300px"),
    )
    button = widgets.Button(
        description="Register model",
        button_style="primary",
    )
    output = widgets.Output()

    def on_click(_: widgets.Button) -> None:
        output.clear_output()
        with output:
            name = name_input.value.strip()
            if not name:
                print("Please enter a model name.")
                return
            model_uri = f"runs:/{run_id}/{model_artifact}"
            result = mlflow.register_model(model_uri, name)
            print(f"Registered '{name}' version {result.version} from run {run_id}")

    button.on_click(on_click)
    display(widgets.HBox([name_input, button]), output)


def show_describe_widget(run_id: str, tracking_uri: str | None = None) -> None:
    """Display a text area + button to add a description to a run.

    The description is stored as the MLflow tag 'mlflow.note.content',
    which is displayed in the MLflow UI under the run's Notes field.

    Args:
        run_id:       MLflow run ID to annotate.
        tracking_uri: MLflow tracking server URI. Uses the active URI if None.
    """
    import mlflow
    import ipywidgets as widgets
    from IPython.display import display

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    desc_input = widgets.Textarea(
        placeholder="Run description...",
        description="Notes:",
        layout=widgets.Layout(width="400px", height="100px"),
    )
    button = widgets.Button(
        description="Save description",
        button_style="primary",
    )
    output = widgets.Output()

    def on_click(_: widgets.Button) -> None:
        output.clear_output()
        with output:
            text = desc_input.value.strip()
            if not text:
                print("Please enter a description.")
                return
            client = mlflow.tracking.MlflowClient()
            client.set_tag(run_id, "mlflow.note.content", text)
            print(f"Description saved to run {run_id}")

    button.on_click(on_click)
    display(widgets.VBox([desc_input, button]), output)
