from uuid import uuid4
from podcasts.notification_service import PodcastItem, PodcastsNotificationService
from podcasts.sources.podcast_init_source import PodcastInitSource
import pytest
from bs4 import BeautifulSoup


TEST_URL = 'https://www.pythonpodcast.com/preql-relational-algebra-sql-replacement-episode-325/'
# TODO: check https://docs.pytest.org/en/latest/example/parametrize.html#apply-indirect-on-particular-arguments
test_uuid = uuid4()


@pytest.fixture
def posts_page_sample():
    """TODO"""
    # with open('podcasts/tests/samples/331_page.html') as f
        # yield f.read()

@pytest.fixture
def post_page_sample():
    with open('podcasts/tests/samples/331_page.html') as f:
        yield f.read()

@pytest.fixture
def source():
    yield PodcastInitSource()


@pytest.fixture
def mocked_source(mocker, source, post_page_sample):
    mocker.patch.object(source, '_find_posts_links', return_value=[TEST_URL], autospec=True)
    mocker.patch.object(source, 'get_page_content', return_value=post_page_sample, autospec=True)
    yield source


@pytest.fixture
def test_item(mocker, source, post_page_sample):
    yield PodcastItem(
        test_uuid=test_uuid,
        url=TEST_URL,
        title='Doing Dask Powered Data Science In The Saturn Cloud - Episode 331',
        source=source.SOURCE_NAME,
    )


def test_create_item(post_page_sample):
    soup = BeautifulSoup(post_page_sample, 'html.parser')
    class_name = 'blog-page-title'
    exptected = 'Doing Dask Powered Data Science In The Saturn Cloud - Episode 331'
    kwargs = {'class': class_name}
    titles = soup.find_all(**kwargs)
    assert titles
    assert titles[0].get_text() == exptected


def test_get_items(mocked_source: PodcastInitSource, test_item):
    count = 1
    items = mocked_source.get_items_after(items_count=count)
    assert len(items) == count
    item = items[0]
    item.item_id = test_uuid
    expected = [test_item]
    assert items == expected


def test_notification(mocked_source: PodcastInitSource, test_item):
    serv = PodcastsNotificationService()
    serv.register_source(mocked_source)
    serv.register_subsciber(lambda x: print(f'from s: {x}'))
    serv.check_new_podcasts()
    assert mocked_source.get_last_processed() == test_item