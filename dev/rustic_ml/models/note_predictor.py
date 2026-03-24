from torch import nn
from rustic_ml.encoding import N_NOTES


class NotePredictor(nn.Module):
    """
    Simple note predictor model. Predicts a MIDI note from a mel
    diagram
    """

    def __init__(self, n_notes: int = N_NOTES):
        super().__init__()

        self.conv1 = nn.Sequential(
            nn.Conv2d(1, 16, kernel_size=3, padding=1),
            nn.BatchNorm2d(16),
            nn.ReLU(),
            nn.MaxPool2d(2)
        )
        self.conv2 = nn.Sequential(
            nn.Conv2d(16, 32, kernel_size=3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(2)
        )

        self.pool = nn.AdaptiveAvgPool2d(1)
        self.dropout = nn.Dropout(0.3)
        self.head = nn.Linear(32, n_notes)
    
        n_params = sum(p.numel() for p in self.parameters() if p.requires_grad)
        print(f"{self.__class__.__name__} - {n_params:,} trainable parameters")

    def forward(self, mel) -> tuple:
        x = self.conv1(mel)
        x = self.conv2(x)
        x = self.pool(x).flatten(start_dim=1)

        return self.head(self.dropout(x))
