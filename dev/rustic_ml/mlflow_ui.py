"""
Interactive IPython widget helpers for MLflow run management.

Provides one-click widgets for registering models and annotating runs,
intended for use inside Jupyter notebooks.
"""


def show_register_widget(run_id: str, tracking_uri: str | None = None) -> None:
    """Display a text field + button to register the model logged in a run.

    The model is expected to have been logged under the artifact path "model"
    (i.e. via mlflow.pytorch.log_model(model, "model")).

    Args:
        run_id:       MLflow run ID whose logged model will be registered.
        tracking_uri: MLflow tracking server URI. Uses the active URI if None.
    """
    import mlflow
    import ipywidgets as widgets
    from IPython.display import display

    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)

    name_input = widgets.Text(
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
            model_uri = f"runs:/{run_id}/model"
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
